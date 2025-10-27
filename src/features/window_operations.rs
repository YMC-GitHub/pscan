// src/features/window_operations.rs
use clap::{Arg, Command};
use crate::cli::SubCommand;
use super::feature_trait::Feature;
use crate::platform::find_windows;
use crate::error::{AppError, AppResult};

/// 窗口操作特性（最大化、最小化、还原）
pub struct WindowOperationsFeature;

impl WindowOperationsFeature {
    pub fn new() -> Self {
        Self
    }
    
    /// 构建最小化子命令
    fn build_minimize_command(&self) -> Command {
        self.build_window_operation_command("windows/minimize", "Minimize windows")
    }
    
    /// 构建最大化子命令
    fn build_maximize_command(&self) -> Command {
        self.build_window_operation_command("windows/maximize", "Maximize windows")
    }
    
    /// 构建还原子命令
    fn build_restore_command(&self) -> Command {
        self.build_window_operation_command("windows/restore", "Restore windows to normal state")
    }
    
    /// 构建窗口操作子命令的通用函数
    fn build_window_operation_command(&self, name: &'static str, about: &'static str) -> Command {
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
    
    /// 统一的字段提取函数
    fn extract_filter_args(matches: &clap::ArgMatches) -> (Option<String>, Option<String>, Option<String>) {
        let pid = matches.get_one::<String>("pid").map(|s| s.to_string());
        let name = matches.get_one::<String>("name").map(|s| s.to_string());
        let title = matches.get_one::<String>("title").map(|s| s.to_string());
        (pid, name, title)
    }
    
    /// 处理窗口操作命令
    fn handle_window_operation(
        &self,
        pid_filter: Option<String>,
        name_filter: Option<String>,
        title_filter: Option<String>,
        all: bool,
        operation: WindowOperation,
    ) -> AppResult<()> {
        // 获取进程名称用于过滤
        let processes = crate::process::get_processes();
        let process_names: Vec<(u32, String)> = processes
            .iter()
            .map(|p| (p.pid.parse().unwrap_or(0), p.name.clone()))
            .collect();

        // 使用平台抽象层查找匹配的窗口
        let windows = find_windows(&pid_filter, &name_filter, &title_filter, &process_names);
        
        // 验证窗口数量
        if windows.is_empty() {
            return Err(AppError::NoMatchingWindows);
        }

        if !all && windows.len() > 1 {
            return Err(AppError::MultipleWindows(windows.len()));
        }

        let mut count = 0;
        for window in windows {
            let result = match operation {
                WindowOperation::Minimize => window.minimize(),
                WindowOperation::Maximize => window.maximize(),
                WindowOperation::Restore => window.restore(),
            };

            match result {
                Ok(()) => {
                    println!("{}: {} (PID: {})", operation.capitalized(), window.title, window.pid);
                    count += 1;
                }
                Err(e) => {
                    eprintln!("Failed to {} window {} (PID: {}): {}", 
                             operation.as_str(), window.title, window.pid, e);
                }
            }
        }

        if count == 0 {
            return Err(AppError::NoWindowsModified);
        }

        println!("Successfully {} {} window(s)", operation.past_tense(), count);
        Ok(())
    }
}

/// 窗口操作类型枚举
#[derive(Debug, Clone, Copy)]
enum WindowOperation {
    Minimize,
    Maximize,
    Restore,
}

impl WindowOperation {
    fn as_str(&self) -> &'static str {
        match self {
            WindowOperation::Minimize => "minimize",
            WindowOperation::Maximize => "maximize",
            WindowOperation::Restore => "restore",
        }
    }
    
    fn past_tense(&self) -> &'static str {
        match self {
            WindowOperation::Minimize => "minimized",
            WindowOperation::Maximize => "maximized",
            WindowOperation::Restore => "restored",
        }
    }
    
    fn capitalized(&self) -> &'static str {
        match self {
            WindowOperation::Minimize => "Minimized",
            WindowOperation::Maximize => "Maximized",
            WindowOperation::Restore => "Restored",
        }
    }
}

impl Feature for WindowOperationsFeature {
    fn name(&self) -> &'static str {
        "window_operations"
    }
    
    fn description(&self) -> &'static str {
        "Window operations (minimize, maximize, restore)"
    }
    
    fn build_cli(&self, command: Command) -> Command {
        command
            .subcommand(self.build_minimize_command())
            .subcommand(self.build_maximize_command())
            .subcommand(self.build_restore_command())
    }
    
    fn parse_cli(&self, matches: &clap::ArgMatches) -> Option<SubCommand> {
        if let Some(matches) = matches.subcommand_matches("windows/minimize") {
            let (pid, name, title) = Self::extract_filter_args(matches);
            let all = matches.get_flag("all");
            Some(SubCommand::WindowsMinimize { pid, name, title, all })
        } else if let Some(matches) = matches.subcommand_matches("windows/maximize") {
            let (pid, name, title) = Self::extract_filter_args(matches);
            let all = matches.get_flag("all");
            Some(SubCommand::WindowsMaximize { pid, name, title, all })
        } else if let Some(matches) = matches.subcommand_matches("windows/restore") {
            let (pid, name, title) = Self::extract_filter_args(matches);
            let all = matches.get_flag("all");
            Some(SubCommand::WindowsRestore { pid, name, title, all })
        } else {
            None
        }
    }
    
    fn execute(&self, subcommand: &SubCommand) -> AppResult<()> {
        match subcommand {
            SubCommand::WindowsMinimize { pid, name, title, all } => {
                self.handle_window_operation(
                    pid.clone(),
                    name.clone(), 
                    title.clone(),
                    *all,
                    WindowOperation::Minimize,
                )
            }
            SubCommand::WindowsMaximize { pid, name, title, all } => {
                self.handle_window_operation(
                    pid.clone(),
                    name.clone(), 
                    title.clone(),
                    *all,
                    WindowOperation::Maximize,
                )
            }
            SubCommand::WindowsRestore { pid, name, title, all } => {
                self.handle_window_operation(
                    pid.clone(),
                    name.clone(), 
                    title.clone(),
                    *all,
                    WindowOperation::Restore,
                )
            }
            _ => Ok(()) // 不是本特性处理的命令，忽略
        }
    }
    
    fn is_supported(&self) -> bool {
        #[cfg(windows)]
        { true }
        #[cfg(not(windows))]
        { false }
    }
}