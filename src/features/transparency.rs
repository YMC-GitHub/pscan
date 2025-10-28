// src/features/transparency.rs
use clap::{Arg, Command};
use crate::cli::SubCommand;
use super::feature_trait::Feature;
use crate::platform::find_windows;
use crate::error::{AppError, AppResult};
use crate::sorting::{SortOrder, PositionSort, apply_window_handle_sorting};
use crate::utils::parse_indices;

/// 窗口透明度特性
pub struct TransparencyFeature;

impl TransparencyFeature {
    pub fn new() -> Self {
        Self
    }
    
    /// 构建子命令
    fn build_command(&self) -> Command {
        Command::new("windows/transparency")
            .about("Set window transparency/opacity level")
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
                Arg::new("level")
                    .short('l')
                    .long("level")
                    .value_name("PERCENT")
                    .num_args(1)
                    .value_parser(clap::value_parser!(u8).range(0..=100))
                    .default_value("100")
                    .help("Transparency level (0-100%, where 100 is fully opaque)")
            )
            .arg(
                Arg::new("reset")
                    .long("reset")
                    .action(clap::ArgAction::SetTrue)
                    .help("Reset transparency to fully opaque (100%)")
                    .conflicts_with("level")
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
    
    /// 处理透明度命令
    fn handle_transparency(
        &self,
        pid_filter: Option<String>,
        name_filter: Option<String>,
        title_filter: Option<String>,
        all: bool,
        index: Option<String>,
        level: u8,
        reset: bool,
        sort_position: PositionSort,
    ) -> AppResult<()> {
        // 确定透明度级别
        let target_level = if reset { 100 } else { level };
        
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

            match window.set_transparency(target_level) {
                Ok(()) => {
                    let action_str = if reset { "reset" } else { "set" };
                    println!("{}: {} (PID: {}) to {}% opacity", 
                             action_str, window.title, window.pid, target_level);
                    count += 1;
                }
                Err(e) => {
                    eprintln!("Failed to set transparency for window {} (PID: {}): {}", 
                             window.title, window.pid, e);
                }
            }
        }

        if count == 0 {
            return Err(AppError::NoWindowsModified);
        }

        println!("Successfully modified {} window(s)", count);
        Ok(())
    }
}

impl Feature for TransparencyFeature {
    fn name(&self) -> &'static str {
        "transparency"
    }
    
    fn description(&self) -> &'static str {
        "Window transparency management"
    }
    
    fn build_cli(&self, command: Command) -> Command {
        command.subcommand(self.build_command())
    }
    
    fn parse_cli(&self, matches: &clap::ArgMatches) -> Option<SubCommand> {
        if let Some(matches) = matches.subcommand_matches("windows/transparency") {
            let (pid, name, title) = Self::extract_filter_args(matches);
            let all = matches.get_flag("all");
            let index = matches.get_one::<String>("index").map(|s| s.to_string());
            let level = *matches.get_one::<u8>("level").unwrap_or(&100);
            let reset = matches.get_flag("reset");
            
            let sort_position = match matches.get_one::<String>("sort_position").map(|s| s.as_str()) {
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
            
            Some(SubCommand::WindowsTransparency { 
                pid, 
                name, 
                title, 
                all,
                index,
                level,
                reset,
                sort_position,
            })
        } else {
            None
        }
    }
    
    fn execute(&self, subcommand: &SubCommand) -> AppResult<()> {
        if let SubCommand::WindowsTransparency { pid, name, title, all, index, level, reset, sort_position } = subcommand {
            self.handle_transparency(
                pid.clone(),
                name.clone(), 
                title.clone(),
                *all,
                index.clone(),
                *level,
                *reset,
                *sort_position,
            )
        } else {
            Ok(()) // 不是本特性处理的命令，忽略
        }
    }
    
    fn is_supported(&self) -> bool {
        #[cfg(windows)]
        { true }
        #[cfg(not(windows))]
        { false }
    }
}