// src/features/window_operations.rs
use clap::{Arg, Command};
use crate::cli::SubCommand;
use super::feature_trait::Feature;
use crate::platform::find_windows;
use crate::error::{AppError, AppResult};
use crate::sorting::{SortOrder, PositionSort, apply_window_handle_sorting};
use crate::utils::parse_indices;

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
            .arg(
                Arg::new("index")
                    .long("index")
                    .value_name("INDICES")
                    .num_args(1)
                    .default_value("")
                    .help("Window indices to set (e.g., \"1,2,3\"), empty means all")
            )
            .arg(
                Arg::new("sort_position")
                    .long("sort-position")
                    .value_name("X_ORDER|Y_ORDER")
                    .num_args(1)
                    .allow_hyphen_values(true)
                    .default_value("0|0")
                    .help("Sort by position: X_ORDER|Y_ORDER, e.g., 1|-1 for X ascending, Y descending")
            )
    }
    
    /// 统一的字段提取函数
    fn extract_filter_args(matches: &clap::ArgMatches) -> (Option<String>, Option<String>, Option<String>) {
        let pid = matches.get_one::<String>("pid").map(|s| s.to_string());
        let name = matches.get_one::<String>("name").map(|s| s.to_string());
        let title = matches.get_one::<String>("title").map(|s| s.to_string());
        (pid, name, title)
    }
    
    /// 解析排序位置参数
    fn parse_sort_position(matches: &clap::ArgMatches) -> PositionSort {
        match matches.get_one::<String>("sort_position").map(|s| s.as_str()) {
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
        }
    }
    
    /// 处理窗口操作命令
    fn handle_window_operation(
        &self,
        pid_filter: Option<String>,
        name_filter: Option<String>,
        title_filter: Option<String>,
        all: bool,
        index: Option<String>,
        operation: WindowOperation,
        sort_position: PositionSort,
    ) -> AppResult<()> {
        // 获取进程名称用于过滤
        let processes = crate::process::get_processes();
        let process_names: Vec<(u32, String)> = processes
            .iter()
            .map(|p| (p.pid.parse().unwrap_or(0), p.name.clone()))
            .collect();

        // 使用平台抽象层查找匹配的窗口
        let mut windows = find_windows(&pid_filter, &name_filter, &title_filter, &process_names);
        
        // 验证窗口数量
        if windows.is_empty() {
            return Err(AppError::NoMatchingWindows);
        }

        // 应用排序
        apply_window_handle_sorting(&mut windows, &SortOrder::None, &sort_position);

        // 解析索引
        let indices = parse_indices(&index.unwrap_or_default(), windows.len());

        let mut count = 0;
        for (i, window) in windows.iter().enumerate() {
            // 检查索引过滤
            if !indices.is_empty() && !indices.contains(&(i + 1)) {
                continue;
            }

            // 检查是否应用所有窗口
            if !all && indices.is_empty() && i > 0 {
                break; // 如果没有指定 --all 且没有指定索引，只操作第一个窗口
            }

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
            let index = matches.get_one::<String>("index").map(|s| s.to_string());
            let sort_position = Self::parse_sort_position(matches);
            Some(SubCommand::WindowsMinimize { pid, name, title, all, index, sort_position })
        } else if let Some(matches) = matches.subcommand_matches("windows/maximize") {
            let (pid, name, title) = Self::extract_filter_args(matches);
            let all = matches.get_flag("all");
            let index = matches.get_one::<String>("index").map(|s| s.to_string());
            let sort_position = Self::parse_sort_position(matches);
            Some(SubCommand::WindowsMaximize { pid, name, title, all, index, sort_position })
        } else if let Some(matches) = matches.subcommand_matches("windows/restore") {
            let (pid, name, title) = Self::extract_filter_args(matches);
            let all = matches.get_flag("all");
            let index = matches.get_one::<String>("index").map(|s| s.to_string());
            let sort_position = Self::parse_sort_position(matches);
            Some(SubCommand::WindowsRestore { pid, name, title, all, index, sort_position })
        } else {
            None
        }
    }
    
    fn execute(&self, subcommand: &SubCommand) -> AppResult<()> {
        match subcommand {
            SubCommand::WindowsMinimize { pid, name, title, all, index, sort_position } => {
                self.handle_window_operation(
                    pid.clone(),
                    name.clone(), 
                    title.clone(),
                    *all,
                    index.clone(),
                    WindowOperation::Minimize,
                    *sort_position,
                )
            }
            SubCommand::WindowsMaximize { pid, name, title, all, index, sort_position } => {
                self.handle_window_operation(
                    pid.clone(),
                    name.clone(), 
                    title.clone(),
                    *all,
                    index.clone(),
                    WindowOperation::Maximize,
                    *sort_position,
                )
            }
            SubCommand::WindowsRestore { pid, name, title, all, index, sort_position } => {
                self.handle_window_operation(
                    pid.clone(),
                    name.clone(), 
                    title.clone(),
                    *all,
                    index.clone(),
                    WindowOperation::Restore,
                    *sort_position,
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