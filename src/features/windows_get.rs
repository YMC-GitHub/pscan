// src/features/windows_get.rs
use clap::{Arg, Command};
use crate::cli::SubCommand;
use super::feature_trait::Feature;
use crate::platform::get_all_windows_with_size;
use crate::process::get_processes;
use crate::output::{OutputFormat, display_windows};
use crate::sorting::{SortOrder, PositionSort, apply_window_sorting};
use crate::error::{AppError, AppResult};

/// 窗口信息获取特性
pub struct WindowsGetFeature;

impl WindowsGetFeature {
    pub fn new() -> Self {
        Self
    }
    
    /// 构建子命令
    fn build_command(&self) -> Command {
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
                    .allow_hyphen_values(true)
                    .value_parser(["1", "-1", "0"])
                    .default_value("0")
                    .help("Sort by PID: 1 (ascending), -1 (descending), 0 (none)")
            )
            .arg(
                Arg::new("sort-position")
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
    
    /// 处理 windows/get 命令
    fn handle_windows_get(
        &self,
        pid_filter: Option<String>,
        name_filter: Option<String>,
        title_filter: Option<String>,
        format: OutputFormat,
        sort_pid: SortOrder,
        sort_position: PositionSort,
    ) -> AppResult<()> {
        // 使用平台抽象层获取所有窗口及其尺寸信息
        let windows = get_all_windows_with_size();
        
        // 获取进程名称用于显示
        let processes = get_processes();
        let process_names: Vec<(u32, String)> = processes
            .iter()
            .map(|p| (p.pid.parse().unwrap_or(0), p.name.clone()))
            .collect();
        
        // 过滤窗口
        let mut filtered_windows: Vec<crate::types::WindowInfo> = windows
            .iter()
            .filter(|window| {
                // PID filter
                if let Some(pid) = &pid_filter {
                    if window.pid.to_string() != *pid {
                        return false;
                    }
                }

                // Name filter
                if let Some(name) = &name_filter {
                    let process_name = process_names
                        .iter()
                        .find(|(process_pid, _)| *process_pid == window.pid)
                        .map(|(_, name)| name.to_lowercase())
                        .unwrap_or_default();
                    
                    if !process_name.contains(&name.to_lowercase()) {
                        return false;
                    }
                }

                // Title filter
                if let Some(title) = &title_filter {
                    if !window.title.to_lowercase().contains(&title.to_lowercase()) {
                        return false;
                    }
                }

                true
            })
            .cloned()
            .collect();

        if filtered_windows.is_empty() {
            return Err(AppError::NoMatchingWindows);
        }

        // 应用排序
        apply_window_sorting(&mut filtered_windows, &sort_pid, &sort_position);

        // 显示结果
        display_windows(&filtered_windows, &process_names, format)
    }
}

impl Feature for WindowsGetFeature {
    fn name(&self) -> &'static str {
        "windows_get"
    }
    
    fn description(&self) -> &'static str {
        "Window information retrieval with filtering and sorting"
    }
    
    fn build_cli(&self, command: Command) -> Command {
        command.subcommand(self.build_command())
    }
    
    fn parse_cli(&self, matches: &clap::ArgMatches) -> Option<SubCommand> {
        if let Some(matches) = matches.subcommand_matches("windows/get") {
            let (pid, name, title) = Self::extract_filter_args(matches);
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
        } else {
            None
        }
    }
    
    fn execute(&self, subcommand: &SubCommand) -> AppResult<()> {
        if let SubCommand::WindowsGet { pid, name, title, format, sort_pid, sort_position } = subcommand {
            self.handle_windows_get(
                pid.clone(),
                name.clone(), 
                title.clone(),
                format.clone(),
                *sort_pid,
                *sort_position,
            )
        } else {
            Ok(()) // 不是本特性处理的命令，忽略
        }
    }
    
    fn is_supported(&self) -> bool {
        // windows/get 功能在所有平台都支持，因为即使没有窗口操作功能，也能获取基本信息
        true
    }
}