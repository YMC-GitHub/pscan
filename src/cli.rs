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
        toggle: bool,
        off: bool,
    },
    WindowsTransparency {  // 新增透明度子命令
        pid: Option<String>,
        name: Option<String>,
        title: Option<String>,
        all: bool,
        level: u8,
        reset: bool,
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
        .arg(
            Arg::new("sort-pid")
                .long("sort-pid")
                .value_name("ORDER")
                .num_args(1)
                .allow_hyphen_values(true)  // 允许以连字符开头的值
                .value_parser(["1", "-1", "0"])
                .default_value("0")
                .help("Sort by PID: 1 (ascending), -1 (descending), 0 (none)")
        )
        .arg(
            Arg::new("sort-position")
                .long("sort-position")
                .value_name("X_ORDER|Y_ORDER")
                .num_args(1)
                .allow_hyphen_values(true)  // 允许以连字符开头的值
                .default_value("0|0")
                .help("Sort by position: X_ORDER|Y_ORDER, e.g., 1|-1 for X ascending, Y descending")
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
                .help("Apply to all matching windows")
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
        
        // 手动解析排序参数
        let sort_pid = match matches.get_one::<String>("sort-pid").map(|s| s.as_str()) {
            Some("1") => SortOrder::Ascending,
            Some("-1") => SortOrder::Descending,
            Some("0") | None => SortOrder::None,
            Some(_) => SortOrder::None, // 不应该发生，因为有 value_parser
        };
        
        let sort_position = match matches.get_one::<String>("sort-position").map(|s| s.as_str()) {
            Some(s) => {
                match s.parse() {
                    Ok(pos) => pos,
                    Err(_) => {
                        eprintln!("Warning: Invalid position sort format '{}', using default", s);
                        PositionSort::default()
                    }
                }
            }
            None => PositionSort::default(),
        };
        
        Some(SubCommand::WindowsGet { 
            pid, 
            name, 
            title, 
            format,
            sort_pid,
            sort_position,
        })
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
    } else {
        None
    }
}

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
    
    // 使用特性管理器构建 CLI
    let matches = feature_manager.build_cli(matches)
        // 保持现有的硬编码子命令（为了向后兼容）
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

    // 优先使用特性管理器解析子命令
    let subcommand = feature_manager.parse_cli(&matches)
        .or_else(|| handle_subcommand_matches(&matches)); // 回退到旧解析逻辑

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