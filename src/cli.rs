use clap::{Arg, Command, Subcommand};
use crate::output::OutputFormat;

pub struct CliConfig {
    pub pid_filter: Option<String>,
    pub name_filter: Option<String>,
    pub title_filter: Option<String>,
    pub has_window_filter: bool,
    pub no_window_filter: bool,
    pub format: OutputFormat,
    pub verbose: bool,
    pub subcommand: Option<SubCommand>,
}

#[derive(Debug)]
pub enum SubCommand {
    WindowsGet {
        pid: Option<String>,
        name: Option<String>,
        title: Option<String>,
        format: OutputFormat,
    },
}

pub fn parse_args() -> CliConfig {
    let matches = Command::new("Process Filter")
        .version(env!("CARGO_PKG_VERSION"))
        .author(env!("CARGO_PKG_AUTHORS"))
        .about(env!("CARGO_PKG_DESCRIPTION"))
        .subcommand_required(false)
        .arg_required_else_help(false)
        .subcommand(
            Command::new("windows/get")
                .about("Get window information including size and position")
                .arg(
                    Arg::new("pid")
                        .short('p')
                        .long("pid")
                        .value_name("PID")
                        .help("Filter by process ID")
                )
                .arg(
                    Arg::new("name")
                        .short('n')
                        .long("name")
                        .value_name("NAME")
                        .help("Filter by process name (contains)")
                )
                .arg(
                    Arg::new("title")
                        .short('t')
                        .long("title")
                        .value_name("TITLE")
                        .help("Filter by window title (contains)")
                )
                .arg(
                    Arg::new("format")
                        .short('f')
                        .long("format")
                        .value_name("FORMAT")
                        .value_parser(clap::value_parser!(OutputFormat))
                        .default_value("table")
                        .help("Output format")
                )
        )
        // 为未来扩展预留
        .subcommand(
            Command::new("windows/set")
                .about("Set window properties (future)")
                .arg(Arg::new("pid").required(true))
        )
        .subcommand(
            Command::new("windows/move")
                .about("Move window position (future)")
                .arg(Arg::new("pid").required(true))
        )
        .arg(
            Arg::new("pid")
                .short('p')
                .long("pid")
                .value_name("PID")
                .help("Filter by process ID")
        )
        .arg(
            Arg::new("name")
                .short('n')
                .long("name")
                .value_name("NAME")
                .help("Filter by process name (contains)")
        )
        .arg(
            Arg::new("title")
                .short('t')
                .long("title")
                .value_name("TITLE")
                .help("Filter by window title (contains)")
        )
        .arg(
            Arg::new("has_window")
                .long("has-window")
                .action(clap::ArgAction::SetTrue)
                .help("Show only processes with windows")
        )
        .arg(
            Arg::new("no_window")
                .long("no-window")
                .action(clap::ArgAction::SetTrue)
                .help("Show only processes without windows")
                .conflicts_with("has_window")
        )
        .arg(
            Arg::new("format")
                .short('f')
                .long("format")
                .value_name("FORMAT")
                .value_parser(clap::value_parser!(OutputFormat))
                .default_value("table")
                .help("Output format")
        )
        .arg(
            Arg::new("verbose")
                .short('v')
                .long("verbose")
                .action(clap::ArgAction::SetTrue)
                .help("Show detailed information")
        )
        .get_matches();

    let subcommand = if let Some(matches) = matches.subcommand_matches("windows/get") {
        Some(SubCommand::WindowsGet {
            pid: matches.get_one::<String>("pid").map(|s| s.to_string()),
            name: matches.get_one::<String>("name").map(|s| s.to_string()),
            title: matches.get_one::<String>("title").map(|s| s.to_string()),
            format: matches.get_one::<OutputFormat>("format").unwrap().clone(),
        })
    } else if let Some(_matches) = matches.subcommand_matches("windows/set") {
        // 处理其他 windows 子命令
        eprintln!("windows/set command is not implemented yet");
        std::process::exit(1);
    } else if let Some(_matches) = matches.subcommand_matches("windows/move") {
        eprintln!("windows/move command is not implemented yet");
        std::process::exit(1);
    } else {
        None
    };

    CliConfig {
        pid_filter: matches.get_one::<String>("pid").map(|s| s.to_string()),
        name_filter: matches.get_one::<String>("name").map(|s| s.to_string()),
        title_filter: matches.get_one::<String>("title").map(|s| s.to_string()),
        has_window_filter: matches.get_flag("has_window"),
        no_window_filter: matches.get_flag("no_window"),
        format: matches.get_one::<OutputFormat>("format").unwrap().clone(),
        verbose: matches.get_flag("verbose"),
        subcommand,
    }
}