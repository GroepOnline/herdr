use std::collections::HashMap;

use crate::api::schema::{TabCreateParams, TabListParams, TabRenameParams};

pub(super) fn run_tab_command(args: &[String]) -> std::io::Result<i32> {
    let Some(subcommand) = args.first().map(|arg| arg.as_str()) else {
        print_tab_help();
        return Ok(2);
    };

    match subcommand {
        "list" => tab_list(&args[1..]),
        "create" => tab_create(&args[1..]),
        "get" => tab_get(&args[1..]),
        "focus" => tab_focus(&args[1..]),
        "rename" => tab_rename(&args[1..]),
        "close" => tab_close(&args[1..]),
        "help" | "--help" | "-h" => {
            print_tab_help();
            Ok(0)
        }
        _ => {
            print_tab_help();
            Ok(2)
        }
    }
}

fn tab_list(args: &[String]) -> std::io::Result<i32> {
    let mut workspace_id = None;

    let mut index = 0;
    while index < args.len() {
        match args[index].as_str() {
            "--workspace" => {
                let Some(value) = args.get(index + 1) else {
                    eprintln!("missing value for --workspace");
                    return Ok(2);
                };
                workspace_id = Some(super::normalize_workspace_id(value));
                index += 2;
            }
            other => {
                eprintln!("unknown option: {other}");
                return Ok(2);
            }
        }
    }

    super::runtime::tab_list(TabListParams { workspace_id })
}

fn tab_create(args: &[String]) -> std::io::Result<i32> {
    let params = match parse_tab_create_args(args) {
        Ok(params) => params,
        Err(message) => {
            eprintln!("{message}");
            return Ok(2);
        }
    };
    super::runtime::tab_create(params)
}

fn parse_tab_create_args(args: &[String]) -> Result<TabCreateParams, String> {
    let separator = args
        .iter()
        .position(|arg| arg == "--")
        .unwrap_or(args.len());
    let mut workspace_id = None;
    let mut cwd = None;
    let mut focus = false;
    let mut label = None;
    let mut env = HashMap::new();
    let mut command = None;

    let mut index = 0;
    while index < separator {
        match args[index].as_str() {
            "--workspace" => {
                let Some(value) = args.get(index + 1) else {
                    return Err("missing value for --workspace".into());
                };
                workspace_id = Some(super::normalize_workspace_id(value));
                index += 2;
            }
            "--cwd" => {
                let Some(value) = args.get(index + 1) else {
                    return Err("missing value for --cwd".into());
                };
                cwd = Some(value.clone());
                index += 2;
            }
            "--label" => {
                let Some(value) = args.get(index + 1) else {
                    return Err("missing value for --label".into());
                };
                label = Some(value.clone());
                index += 2;
            }
            "--focus" => {
                focus = true;
                index += 1;
            }
            "--no-focus" => {
                focus = false;
                index += 1;
            }
            "--env" => {
                let Some(value) = args.get(index + 1) else {
                    return Err("missing value for --env".into());
                };
                let (key, value) = super::parse_env_assignment(value)?;
                env.insert(key, value);
                index += 2;
            }
            "--argv" | "--command" => {
                if command.is_some() {
                    return Err("command specified more than once".into());
                }
                let mut argv = Vec::new();
                index += 1;
                while index < separator {
                    let arg = &args[index];
                    if arg.starts_with("--") && is_tab_create_option(arg) {
                        break;
                    }
                    argv.push(arg.clone());
                    index += 1;
                }
                if argv.is_empty() {
                    return Err("missing value for --argv".into());
                }
                command = Some(argv);
            }
            other => return Err(format!("unknown option: {other}")),
        }
    }

    if separator < args.len() {
        if command.is_some() {
            return Err("cannot combine --argv with '--' command separator".into());
        }
        let cmd = args[separator + 1..].to_vec();
        if cmd.is_empty() {
            return Err("missing command after --".into());
        }
        command = Some(cmd);
    }

    Ok(TabCreateParams {
        command,
        workspace_id,
        cwd,
        focus,
        label,
        env,
    })
}

fn is_tab_create_option(arg: &str) -> bool {
    matches!(
        arg,
        "--workspace"
            | "--cwd"
            | "--label"
            | "--focus"
            | "--no-focus"
            | "--env"
            | "--argv"
            | "--command"
    )
}

fn tab_get(args: &[String]) -> std::io::Result<i32> {
    let Some(raw_tab_id) = args.first() else {
        eprintln!("usage: herdr tab get <tab_id>");
        return Ok(2);
    };
    if args.len() != 1 {
        eprintln!("usage: herdr tab get <tab_id>");
        return Ok(2);
    }

    super::runtime::tab_get(super::normalize_tab_id(raw_tab_id))
}

fn tab_focus(args: &[String]) -> std::io::Result<i32> {
    let Some(raw_tab_id) = args.first() else {
        eprintln!("usage: herdr tab focus <tab_id>");
        return Ok(2);
    };
    if args.len() != 1 {
        eprintln!("usage: herdr tab focus <tab_id>");
        return Ok(2);
    }

    super::runtime::tab_focus(super::normalize_tab_id(raw_tab_id))
}

fn tab_rename(args: &[String]) -> std::io::Result<i32> {
    if args.len() < 2 {
        eprintln!("usage: herdr tab rename <tab_id> <label>");
        return Ok(2);
    }

    super::runtime::tab_rename(TabRenameParams {
        tab_id: super::normalize_tab_id(&args[0]),
        label: args[1..].join(" "),
    })
}

fn tab_close(args: &[String]) -> std::io::Result<i32> {
    let Some(raw_tab_id) = args.first() else {
        eprintln!("usage: herdr tab close <tab_id>");
        return Ok(2);
    };
    if args.len() != 1 {
        eprintln!("usage: herdr tab close <tab_id>");
        return Ok(2);
    }

    super::runtime::tab_close(super::normalize_tab_id(raw_tab_id))
}

fn print_tab_help() {
    eprintln!("herdr tab commands:");
    eprintln!("  herdr tab list [--workspace <workspace_id>]");
    eprintln!(
        "  herdr tab create [--workspace <workspace_id>] [--cwd PATH] [--label TEXT] [--env KEY=VALUE] [--argv COMMAND...|-- <command...>] [--focus] [--no-focus]"
    );
    eprintln!("  herdr tab get <tab_id>");
    eprintln!("  herdr tab focus <tab_id>");
    eprintln!("  herdr tab rename <tab_id> <label>");
    eprintln!("  herdr tab close <tab_id>");
}

#[cfg(test)]
mod tests {
    use super::*;

    fn args(values: &[&str]) -> Vec<String> {
        values.iter().map(|value| (*value).to_string()).collect()
    }

    #[test]
    fn parse_tab_create_args_argv_does_not_swallow_later_flags() {
        let params = parse_tab_create_args(&args(&[
            "--argv",
            "bash",
            "-lc",
            "echo hi",
            "--cwd",
            "/tmp",
        ]))
        .unwrap();

        assert_eq!(
            params.command,
            Some(vec!["bash".into(), "-lc".into(), "echo hi".into()])
        );
        assert_eq!(params.cwd, Some("/tmp".into()));
    }

    #[test]
    fn parse_tab_create_args_accepts_command_after_separator() {
        let params = parse_tab_create_args(&args(&[
            "--label",
            "run",
            "--",
            "cargo",
            "test",
        ]))
        .unwrap();

        assert_eq!(
            params.command,
            Some(vec!["cargo".into(), "test".into()])
        );
        assert_eq!(params.label, Some("run".into()));
    }

    #[test]
    fn parse_tab_create_args_rejects_combining_argv_and_separator() {
        let err = parse_tab_create_args(&args(&["--argv", "bash", "--", "zsh"])).unwrap_err();
        assert!(err.contains("cannot combine"));
    }
}
