use std::io;
use std::time::Duration;

pub fn run_browser_command(args: &[String]) -> io::Result<i32> {
    let mut wait = false;
    let mut url = None;
    let mut index = 0;
    while index < args.len() {
        match args[index].as_str() {
            "--wait" => {
                wait = true;
                index += 1;
            }
            "--help" | "-h" => {
                print_browser_help();
                return Ok(0);
            }
            other if other.starts_with('-') => {
                eprintln!("unknown option: {other}");
                print_browser_help();
                return Ok(2);
            }
            other => {
                if url.is_some() {
                    eprintln!("unexpected extra argument: {other}");
                    return Ok(2);
                }
                url = Some(other.to_string());
                index += 1;
            }
        }
    }

    let Some(url) = url else {
        eprintln!("Missing URL");
        print_browser_help();
        return Ok(2);
    };

    let url = match normalize_browser_url(&url) {
        Ok(url) => url,
        Err(message) => {
            eprintln!("{message}");
            return Ok(1);
        }
    };

    if let Err(e) = run_headless(&url, wait) {
        eprintln!("Browser error: {e}");
        return Ok(1);
    }
    Ok(0)
}

fn print_browser_help() {
    eprintln!("Render a webpage in the terminal using headless Chrome and viuer.");
    eprintln!();
    eprintln!("Requires Google Chrome or Chromium installed and discoverable on PATH.");
    eprintln!();
    eprintln!("usage: herdr browser [--wait] <URL>");
    eprintln!();
    eprintln!("Options:");
    eprintln!("  --wait  Wait for Enter before exiting (default: exit after render)");
}

fn normalize_browser_url(raw: &str) -> Result<String, String> {
    let trimmed = raw.trim();
    if trimmed.is_empty() {
        return Err("URL must not be empty".into());
    }

    let (scheme, rest) = if let Some(idx) = trimmed.find("://") {
        (
            trimmed[..idx].to_ascii_lowercase(),
            trimmed[idx + 3..].to_string(),
        )
    } else if trimmed.contains(':') && !trimmed.starts_with('[') {
        let scheme = trimmed
            .split_once(':')
            .map(|(scheme, _)| scheme.to_ascii_lowercase())
            .unwrap_or_default();
        return Err(format!(
            "unsupported URL scheme: {scheme} (only http and https are allowed)"
        ));
    } else {
        ("http".to_string(), trimmed.to_string())
    };

    match scheme.as_str() {
        "http" | "https" => {}
        other => {
            return Err(format!(
                "unsupported URL scheme: {other} (only http and https are allowed)"
            ));
        }
    }

    if rest.is_empty() {
        return Err("URL must include a host".into());
    }

    Ok(format!("{scheme}://{rest}"))
}

fn run_headless(url: &str, wait: bool) -> anyhow::Result<()> {
    use headless_chrome::protocol::cdp::Page::CaptureScreenshotFormatOption;
    use headless_chrome::{Browser, LaunchOptionsBuilder};
    use image::load_from_memory;

    let launch_options = LaunchOptionsBuilder::default()
        .ignore_certificate_errors(false)
        .build()
        .map_err(|err| anyhow::anyhow!("failed to configure browser launch: {err}"))?;
    let browser = Browser::new(launch_options)?;
    browser.set_default_timeout(Duration::from_secs(30));

    let tab = browser.new_tab()?;

    tab.navigate_to(url)?;
    tab.wait_until_navigated()?;

    let jpeg_data =
        tab.capture_screenshot(CaptureScreenshotFormatOption::Jpeg, None, None, true)?;

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

    viuer::print(&img, &conf).map_err(|e| anyhow::anyhow!("viuer error: {e}"))?;

    if wait {
        eprintln!("\nPress Enter to exit browser...");
        let mut line = String::new();
        let _ = io::stdin().read_line(&mut line);
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::normalize_browser_url;

    #[test]
    fn normalize_browser_url_adds_http_scheme() {
        assert_eq!(
            normalize_browser_url("example.com").unwrap(),
            "http://example.com"
        );
    }

    #[test]
    fn normalize_browser_url_accepts_http_and_https_case_insensitively() {
        assert_eq!(
            normalize_browser_url("HTTPS://Example.COM/path").unwrap(),
            "https://Example.COM/path"
        );
        assert_eq!(
            normalize_browser_url("HTTP://localhost:8080").unwrap(),
            "http://localhost:8080"
        );
    }

    #[test]
    fn normalize_browser_url_rejects_non_http_schemes() {
        assert!(normalize_browser_url("file:///etc/passwd")
            .unwrap_err()
            .contains("unsupported URL scheme"));
        assert!(normalize_browser_url("javascript:alert(1)")
            .unwrap_err()
            .contains("unsupported URL scheme"));
    }
}
