// src/cli.rs
use clap::{Arg, Command};
use crate::output::OutputFormat;
use crate::sorting::{SortOrder, PositionSort};  // 从 sorting 模块导入
use crate::features;

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
        sort_pid: SortOrder,
        sort_position: PositionSort,
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
    WindowsPositionSet {
        pid: Option<String>,
        name: Option<String>,
        title: Option<String>,
        all: bool,
        position: Option<String>,
        index: Option<String>,
        layout: Option<String>,
        x_start: Option<String>,
        y_start: Option<String>,
        x_step: Option<String>,
        y_step: Option<String>,
        sort_position: PositionSort,
    },
    WindowsAlwaysOnTop {
        pid: Option<String>,
        name: Option<String>,
        title: Option<String>,
        all: bool,
        index: Option<String>,
        toggle: bool,
        off: bool,
        sort_position: PositionSort,
    },
    WindowsTransparency {
        pid: Option<String>,
        name: Option<String>,
        title: Option<String>,
        all: bool,
        index: Option<String>,
        level: u8,
        reset: bool,
        sort_position: PositionSort,
    },
}

// 删除原来的 SortOrder 和 PositionSort 定义，因为它们已移动到 sorting.rs

// 统一的字段提取函数
fn extract_filter_args(matches: &clap::ArgMatches) -> (Option<String>, Option<String>, Option<String>) {
    let pid = matches.get_one::<String>("pid").map(|s| s.to_string());
    let name = matches.get_one::<String>("name").map(|s| s.to_string());
    let title = matches.get_one::<String>("title").map(|s| s.to_string());
    (pid, name, title)
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

// 删除原来的 build_windows_get_command 和 handle_subcommand_matches 函数

pub fn parse_args() -> CliConfig {
    let feature_manager = features::create_default_manager();
    
    let matches = build_common_args(
        Command::new("Process Filter")
            .version(env!("CARGO_PKG_VERSION"))
            .author(env!("CARGO_PKG_AUTHORS"))
            .about(env!("CARGO_PKG_DESCRIPTION"))
            .subcommand_required(false)
            .arg_required_else_help(false)
    );
    
    // 使用特性管理器构建 CLI（现在包含所有窗口操作命令）
    let matches = feature_manager.build_cli(matches)
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

    // 完全使用特性管理器解析子命令
    let subcommand = feature_manager.parse_cli(&matches);

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