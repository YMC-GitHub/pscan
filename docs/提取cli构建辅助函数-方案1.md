基于您的代码，我来提取CLI构建辅助函数，为每个subcommand创建构建函数，并统一字段提取逻辑：

```rust
// src/cli.rs
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

// 通用的窗口过滤器参数构建函数
fn window_filter_args() -> Vec<Arg> {
    vec![
        Arg::new("pid")
            .short('p')
            .long("pid")
            .value_name("PID")
            .help("Filter by process ID"),
        Arg::new("name")
            .short('n')
            .long("name")
            .value_name("NAME")
            .help("Filter by process name (contains)"),
        Arg::new("title")
            .short('t')
            .long("title")
            .value_name("TITLE")
            .help("Filter by window title (contains)"),
    ]
}

// 通用的输出格式参数构建函数
fn output_format_arg() -> Arg {
    Arg::new("format")
        .short('f')
        .long("format")
        .value_name("FORMAT")
        .value_parser(clap::value_parser!(OutputFormat))
        .default_value("table")
        .help("Output format")
}

// 通用的all参数构建函数
fn all_flag_arg() -> Arg {
    Arg::new("all")
        .short('a')
        .long("all")
        .action(clap::ArgAction::SetTrue)
        .help("Apply to all matching windows")
}

// 子命令构建函数
fn build_windows_get_command() -> Command {
    let mut cmd = Command::new("windows/get")
        .about("Get window information including size and position");
    
    for arg in window_filter_args() {
        cmd = cmd.arg(arg);
    }
    
    cmd.arg(output_format_arg())
}

fn build_windows_minimize_command() -> Command {
    let mut cmd = Command::new("windows/minimize")
        .about("Minimize windows");
    
    for arg in window_filter_args() {
        cmd = cmd.arg(arg);
    }
    
    cmd.arg(all_flag_arg())
}

fn build_windows_maximize_command() -> Command {
    let mut cmd = Command::new("windows/maximize")
        .about("Maximize windows");
    
    for arg in window_filter_args() {
        cmd = cmd.arg(arg);
    }
    
    cmd.arg(all_flag_arg())
}

fn build_windows_restore_command() -> Command {
    let mut cmd = Command::new("windows/restore")
        .about("Restore windows to normal state");
    
    for arg in window_filter_args() {
        cmd = cmd.arg(arg);
    }
    
    cmd.arg(all_flag_arg())
}

fn build_future_commands() -> Vec<Command> {
    vec![
        Command::new("windows/set")
            .about("Set window properties (future)")
            .arg(Arg::new("pid").required(true)),
        Command::new("windows/move")
            .about("Move window position (future)")
            .arg(Arg::new("pid").required(true)),
    ]
}

// 通用的字段提取函数
fn extract_window_filter_fields(matches: &clap::ArgMatches) -> (Option<String>, Option<String>, Option<String>) {
    let pid = matches.get_one::<String>("pid").map(|s| s.to_string());
    let name = matches.get_one::<String>("name").map(|s| s.to_string());
    let title = matches.get_one::<String>("title").map(|s| s.to_string());
    (pid, name, title)
}

fn extract_subcommand_windows_get(matches: &clap::ArgMatches) -> SubCommand {
    let (pid, name, title) = extract_window_filter_fields(matches);
    let format = matches.get_one::<OutputFormat>("format").unwrap().clone();
    
    SubCommand::WindowsGet {
        pid,
        name,
        title,
        format,
    }
}

fn extract_subcommand_windows_minimize(matches: &clap::ArgMatches) -> SubCommand {
    let (pid, name, title) = extract_window_filter_fields(matches);
    let all = matches.get_flag("all");
    
    SubCommand::WindowsMinimize {
        pid,
        name,
        title,
        all,
    }
}

fn extract_subcommand_windows_maximize(matches: &clap::ArgMatches) -> SubCommand {
    let (pid, name, title) = extract_window_filter_fields(matches);
    let all = matches.get_flag("all");
    
    SubCommand::WindowsMaximize {
        pid,
        name,
        title,
        all,
    }
}

fn extract_subcommand_windows_restore(matches: &clap::ArgMatches) -> SubCommand {
    let (pid, name, title) = extract_window_filter_fields(matches);
    let all = matches.get_flag("all");
    
    SubCommand::WindowsRestore {
        pid,
        name,
        title,
        all,
    }
}

// 子命令分发函数
fn handle_subcommand(matches: &clap::ArgMatches) -> Option<SubCommand> {
    match matches.subcommand() {
        Some(("windows/get", sub_matches)) => Some(extract_subcommand_windows_get(sub_matches)),
        Some(("windows/minimize", sub_matches)) => Some(extract_subcommand_windows_minimize(sub_matches)),
        Some(("windows/maximize", sub_matches)) => Some(extract_subcommand_windows_maximize(sub_matches)),
        Some(("windows/restore", sub_matches)) => Some(extract_subcommand_windows_restore(sub_matches)),
        Some(("windows/set", _)) => {
            eprintln!("windows/set command is not implemented yet");
            std::process::exit(1);
        }
        Some(("windows/move", _)) => {
            eprintln!("windows/move command is not implemented yet");
            std::process::exit(1);
        }
        _ => None,
    }
}

pub fn parse_args() -> CliConfig {
    let mut app = Command::new("Process Filter")
        .version(env!("CARGO_PKG_VERSION"))
        .author(env!("CARGO_PKG_AUTHORS"))
        .about(env!("CARGO_PKG_DESCRIPTION"))
        .subcommand_required(false)
        .arg_required_else_help(false);

    // 添加子命令
    app = app.subcommand(build_windows_get_command());
    app = app.subcommand(build_windows_minimize_command());
    app = app.subcommand(build_windows_maximize_command());
    app = app.subcommand(build_windows_restore_command());
    
    // 添加未来命令
    for cmd in build_future_commands() {
        app = app.subcommand(cmd);
    }

    // 添加全局参数
    for arg in window_filter_args() {
        app = app.arg(arg);
    }
    
    app = app.arg(
        Arg::new("has_window")
            .long("has-window")
            .action(clap::ArgAction::SetTrue)
            .help("Show only processes with windows")
    );
    
    app = app.arg(
        Arg::new("no_window")
            .long("no-window")
            .action(clap::ArgAction::SetTrue)
            .help("Show only processes without windows")
            .conflicts_with("has_window")
    );
    
    app = app.arg(output_format_arg());
    
    app = app.arg(
        Arg::new("verbose")
            .short('v')
            .long("verbose")
            .action(clap::ArgAction::SetTrue)
            .help("Show detailed information")
    );

    let matches = app.get_matches();

    let subcommand = handle_subcommand(&matches);

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
```

## 主要改进点：

1. **提取通用参数构建函数**：
   - `window_filter_args()` - 通用的窗口过滤器参数
   - `output_format_arg()` - 输出格式参数
   - `all_flag_arg()` - all标志参数

2. **子命令构建函数**：
   - `build_windows_*_command()` - 每个子命令的构建逻辑
   - 使用通用参数避免重复代码

3. **统一字段提取**：
   - `extract_window_filter_fields()` - 提取通用的窗口过滤字段
   - `extract_subcommand_*()` - 每个子命令的字段提取

4. **子命令分发**：
   - `handle_subcommand()` - 统一的子命令分发逻辑
   - 使用模式匹配替代冗长的if-else链

## 优势：

- **减少重复代码**：窗口过滤器参数现在只需定义一次
- **提高可维护性**：修改参数时只需修改一个地方
- **更好的结构**：每个函数职责单一，易于理解和测试
- **易于扩展**：添加新的子命令时模式清晰

这种重构显著降低了CLI模块的复杂度，使代码更易于维护和扩展。