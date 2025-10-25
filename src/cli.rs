use clap::{Arg, Command};
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
    WindowsMinimize {
        pid: Option<String>,
        name: Option<String>,
        title: Option<String>,
        all: bool,
    },
    WindowsMaximize {
        pid: Option<String>,
        name: Option<String>,
        title: Option<String>,
        all: bool,
    },
    WindowsRestore {
        pid: Option<String>,
        name: Option<String>,
        title: Option<String>,
        all: bool,
    },
}

// 统一的字段提取函数
fn extract_filter_args(matches: &clap::ArgMatches) -> (Option<String>, Option<String>, Option<String>) {
    let pid = matches.get_one::<String>("pid").map(|s| s.to_string());
    let name = matches.get_one::<String>("name").map(|s| s.to_string());
    let title = matches.get_one::<String>("title").map(|s| s.to_string());
    (pid, name, title)
}

// 构建windows/get子命令
fn build_windows_get_command() -> Command {
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
}

// 构建窗口操作子命令的通用函数
fn build_window_operation_command(name: &'static str, about: &'static str) -> Command {
    Command::new(name)
        .about(about)
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
            Arg::new("all")
                .short('a')
                .long("all")
                .action(clap::ArgAction::SetTrue)
                .help(format!("{} all matching windows", about.split_whitespace().next().unwrap_or("Operate on")))
        )
}

// 构建主命令的通用参数
fn build_common_args(command: Command) -> Command {
    command
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
}

// 处理子命令匹配的辅助函数
fn handle_subcommand_matches(matches: &clap::ArgMatches) -> Option<SubCommand> {
    if let Some(matches) = matches.subcommand_matches("windows/get") {
        let (pid, name, title) = extract_filter_args(matches);
        let format = matches.get_one::<OutputFormat>("format").unwrap().clone();
        Some(SubCommand::WindowsGet { pid, name, title, format })
    } else if let Some(matches) = matches.subcommand_matches("windows/minimize") {
        let (pid, name, title) = extract_filter_args(matches);
        let all = matches.get_flag("all");
        Some(SubCommand::WindowsMinimize { pid, name, title, all })
    } else if let Some(matches) = matches.subcommand_matches("windows/maximize") {
        let (pid, name, title) = extract_filter_args(matches);
        let all = matches.get_flag("all");
        Some(SubCommand::WindowsMaximize { pid, name, title, all })
    } else if let Some(matches) = matches.subcommand_matches("windows/restore") {
        let (pid, name, title) = extract_filter_args(matches);
        let all = matches.get_flag("all");
        Some(SubCommand::WindowsRestore { pid, name, title, all })
    } else if let Some(_matches) = matches.subcommand_matches("windows/set") {
        eprintln!("windows/set command is not implemented yet");
        std::process::exit(1);
    } else if let Some(_matches) = matches.subcommand_matches("windows/move") {
        eprintln!("windows/move command is not implemented yet");
        std::process::exit(1);
    } else {
        None
    }
}

pub fn parse_args() -> CliConfig {
    let matches = build_common_args(
        Command::new("Process Filter")
            .version(env!("CARGO_PKG_VERSION"))
            .author(env!("CARGO_PKG_AUTHORS"))
            .about(env!("CARGO_PKG_DESCRIPTION"))
            .subcommand_required(false)
            .arg_required_else_help(false)
    )
    .subcommand(build_windows_get_command())
    .subcommand(build_window_operation_command("windows/minimize", "Minimize windows"))
    .subcommand(build_window_operation_command("windows/maximize", "Maximize windows"))
    .subcommand(build_window_operation_command("windows/restore", "Restore windows to normal state"))
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
    .get_matches();

    let subcommand = handle_subcommand_matches(&matches);

    let (pid_filter, name_filter, title_filter) = extract_filter_args(&matches);
    
    CliConfig {
        pid_filter,
        name_filter,
        title_filter,
        has_window_filter: matches.get_flag("has_window"),
        no_window_filter: matches.get_flag("no_window"),
        format: matches.get_one::<OutputFormat>("format").unwrap().clone(),
        verbose: matches.get_flag("verbose"),
        subcommand,
    }
}