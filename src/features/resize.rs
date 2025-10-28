// src/features/resize.rs
use clap::{Arg, Command};
use crate::cli::SubCommand;
use super::feature_trait::Feature;
use crate::platform::find_windows;
use crate::error::{AppError, AppResult};
use crate::sorting::{SortOrder, PositionSort, apply_window_handle_sorting};
use crate::utils::parse_indices;

/// 窗口调整大小特性
pub struct ResizeFeature;

impl ResizeFeature {
    pub fn new() -> Self {
        Self
    }
    
    /// 构建子命令
    fn build_command(&self) -> Command {
        Command::new("windows/resize")
            .about("Resize windows to specified dimensions")
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
                    .help("Window indices to resize (e.g., \"1,2,3\"), empty means all")
            )
            .arg(
                Arg::new("width")
                    .short('W')  // 使用大写 W 而不是小写 h
                    .long("width")
                    .value_name("WIDTH")
                    .num_args(1)
                    .required_unless_present("size")
                    .help("Window width in pixels")
            )
            .arg(
                Arg::new("height")
                    .short('H')  // 使用大写 H 而不是小写 h
                    .long("height")
                    .value_name("HEIGHT")
                    .num_args(1)
                    .required_unless_present("size")
                    .help("Window height in pixels")
            )
            .arg(
                Arg::new("size")
                    .long("size")
                    .value_name("WIDTHxHEIGHT")
                    .num_args(1)
                    .help("Window size in format WIDTHxHEIGHT (e.g., \"800x600\")")
                    .conflicts_with_all(["width", "height"])
            )
            .arg(
                Arg::new("keep-position")
                    .long("keep-position")
                    .action(clap::ArgAction::SetTrue)
                    .help("Keep current window position, only change size")
            )
            .arg(
                Arg::new("center")
                    .long("center")
                    .action(clap::ArgAction::SetTrue)
                    .help("Center window on screen after resizing")
                    .conflicts_with("keep-position")
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
    
    /// 解析尺寸字符串 "WIDTHxHEIGHT" -> (width, height)
    fn parse_size(size_str: &str) -> AppResult<(i32, i32)> {
        let parts: Vec<&str> = size_str.split('x').collect();
        if parts.len() != 2 {
            return Err(AppError::parse(format!("Invalid size format: {}. Expected 'WIDTHxHEIGHT'", size_str)));
        }

        let width = parts[0].trim().parse()
            .map_err(|_| AppError::parse(format!("Invalid width: {}", parts[0])))?;
        let height = parts[1].trim().parse()
            .map_err(|_| AppError::parse(format!("Invalid height: {}", parts[1])))?;

        if width <= 0 || height <= 0 {
            return Err(AppError::invalid_parameter("Width and height must be positive values"));
        }

        Ok((width, height))
    }
    
    /// 处理调整大小命令
    fn handle_resize(
        &self,
        pid_filter: Option<String>,
        name_filter: Option<String>,
        title_filter: Option<String>,
        all: bool,
        index: Option<String>,
        width: Option<String>,
        height: Option<String>,
        size: Option<String>,
        keep_position: bool,
        center: bool,
        sort_position: PositionSort,
    ) -> AppResult<()> {
        // 解析尺寸参数
        let (target_width, target_height) = if let Some(size_str) = size {
            Self::parse_size(&size_str)?
        } else {
            let w = width.ok_or_else(|| AppError::invalid_parameter("Width is required"))?
                .parse()
                .map_err(|_| AppError::parse("Invalid width value"))?;
            let h = height.ok_or_else(|| AppError::invalid_parameter("Height is required"))?
                .parse()
                .map_err(|_| AppError::parse("Invalid height value"))?;
            
            if w <= 0 || h <= 0 {
                return Err(AppError::invalid_parameter("Width and height must be positive values"));
            }
            
            (w, h)
        };
        
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

            // 执行调整大小操作
            match window.resize(target_width, target_height, keep_position, center) {
                Ok(()) => {
                    println!("Resized: {} (PID: {}) to {}x{}", 
                             window.title, window.pid, target_width, target_height);
                    count += 1;
                }
                Err(e) => {
                    eprintln!("Failed to resize window {} (PID: {}): {}", 
                             window.title, window.pid, e);
                }
            }
        }

        if count == 0 {
            return Err(AppError::NoWindowsModified);
        }

        println!("Successfully resized {} window(s)", count);
        Ok(())
    }
}

impl Feature for ResizeFeature {
    fn name(&self) -> &'static str {
        "resize"
    }
    
    fn description(&self) -> &'static str {
        "Window resizing functionality"
    }
    
    fn build_cli(&self, command: Command) -> Command {
        command.subcommand(self.build_command())
    }
    
    fn parse_cli(&self, matches: &clap::ArgMatches) -> Option<SubCommand> {
        if let Some(matches) = matches.subcommand_matches("windows/resize") {
            let (pid, name, title) = Self::extract_filter_args(matches);
            let all = matches.get_flag("all");
            let index = matches.get_one::<String>("index").map(|s| s.to_string());
            let width = matches.get_one::<String>("width").map(|s| s.to_string());
            let height = matches.get_one::<String>("height").map(|s| s.to_string());
            let size = matches.get_one::<String>("size").map(|s| s.to_string());
            let keep_position = matches.get_flag("keep-position");
            let center = matches.get_flag("center");
            
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
            
            Some(SubCommand::WindowsResize { 
                pid, 
                name, 
                title, 
                all,
                index,
                width,
                height,
                size,
                keep_position,
                center,
                sort_position,
            })
        } else {
            None
        }
    }
    
    fn execute(&self, subcommand: &SubCommand) -> AppResult<()> {
        if let SubCommand::WindowsResize { 
            pid, name, title, all, index, width, height, size, 
            keep_position, center, sort_position 
        } = subcommand {
            self.handle_resize(
                pid.clone(),
                name.clone(), 
                title.clone(),
                *all,
                index.clone(),
                width.clone(),
                height.clone(),
                size.clone(),
                *keep_position,
                *center,
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