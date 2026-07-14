use std::io;

pub fn run_browser_command(args: &[String]) -> io::Result<i32> {
    if args.is_empty() {
        eprintln!("Missing URL");
        return Ok(1);
    }
    let url = if !args[0].starts_with("http") {
        format!("http://{}", args[0])
    } else {
        args[0].clone()
    };

    if let Err(e) = run_headless(&url) {
        eprintln!("Browser error: {}", e);
        return Ok(1);
    }
    Ok(0)
}

fn run_headless(url: &str) -> anyhow::Result<()> {
    use headless_chrome::Browser;
    use headless_chrome::protocol::cdp::Page::CaptureScreenshotFormatOption;
    use image::load_from_memory;

    let browser = Browser::default()?;
    let tab = browser.new_tab()?;
    
    tab.navigate_to(url)?;
    tab.wait_until_navigated()?;
    
    let jpeg_data = tab.capture_screenshot(
        CaptureScreenshotFormatOption::Jpeg,
        None,
        None,
        true,
    )?;
    
    let img = load_from_memory(&jpeg_data)?;
    
    let conf = viuer::Config {
        transparent: false,
        absolute_offset: false,
        x: 0,
        y: 0,
        restore_cursor: false,
        width: None,
        height: None,
        truecolor: true,
        use_kitty: false,
        use_iterm: false,
        premultiplied_alpha: false,
    };
    
    viuer::print(&img, &conf).map_err(|e| anyhow::anyhow!("viuer error: {}", e))?;
    
    println!("\nPress Enter to exit browser...");
    let mut s = String::new();
    let _ = std::io::stdin().read_line(&mut s);
    
    Ok(())
}
