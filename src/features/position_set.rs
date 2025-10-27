// src/features/position_set.rs
use clap::{Arg, Command};
use crate::cli::SubCommand;
use super::feature_trait::Feature;
use crate::platform::find_windows;
use crate::error::{AppError, AppResult};
use crate::sorting::{SortOrder, PositionSort};
use crate::utils::{parse_indices, validate_position_parameters, calculate_positions};

/// 窗口位置设置特性
pub struct PositionSetFeature;

impl PositionSetFeature {
    pub fn new() -> Self {
        Self
    }
    
    /// 构建子命令
    fn build_command(&self) -> Command {
        Command::new("windows/position/set")
            .about("Set window position with various layout options")
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
                Arg::new("position")
                    .long("position")
                    .value_name("X,Y")
                    .num_args(1)
                    .help("Set window position (e.g., \"100,100\")")
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
                Arg::new("layout")
                    .long("layout")
                    .value_name("POSITIONS")
                    .num_args(1)
                    .default_value("")
                    .help("Multiple positions layout (e.g., \"100,100,150,120,200,140\")")
            )
            .arg(
                Arg::new("x_start")
                    .long("x-start")
                    .value_name("X")
                    .num_args(1)
                    .help("Starting X position for multiple windows")
            )
            .arg(
                Arg::new("y_start")
                    .long("y-start")
                    .value_name("Y")
                    .num_args(1)
                    .help("Starting Y position for multiple windows")
            )
            .arg(
                Arg::new("x_step")
                    .long("x-step")
                    .value_name("STEP")
                    .num_args(1)
                    .help("X step for multiple windows")
            )
            .arg(
                Arg::new("y_step")
                    .long("y-step")
                    .value_name("STEP")
                    .num_args(1)
                    .help("Y step for multiple windows")
            )
            .arg(
                Arg::new("sort_position")
                    .long("sort-position")
                    .value_name("X_ORDER|Y_ORDER")
                    .num_args(1)
                    .allow_hyphen_values(true)
                    .default_value("1|1")
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
    
    /// 处理位置设置命令
    fn handle_position_set(
        &self,
        pid_filter: Option<String>,
        name_filter: Option<String>,
        title_filter: Option<String>,
        all: bool,
        position: Option<String>,
        index: Option<String>,
        layout: Option<String>,
        x_start: Option<String>,
        y_start: Option<String>,
        x_step: Option<String>,
        y_step: Option<String>,
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
        crate::sorting::apply_window_handle_sorting(&mut windows, &SortOrder::None, &sort_position);

        // 解析索引
        let indices = parse_indices(&index.unwrap_or_default(), windows.len());
        
        // 验证参数组合
        validate_position_parameters(&position, &layout, &x_start, &y_start, &x_step, &y_step)?;

        // 获取位置列表
        let positions = calculate_positions(
            windows.len(),
            &position,
            &layout.unwrap_or_default(),
            &x_start, &y_start, &x_step, &y_step,
        )?;

        // 执行位置设置
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

            // 获取对应的位置
            if let Some(pos) = positions.get(i) {
                match window.set_position(pos.0, pos.1) {
                    Ok(()) => {
                        println!("{}: {} (PID: {}) to position {},{}", 
                                 "Position set", window.title, window.pid, pos.0, pos.1);
                        count += 1;
                    }
                    Err(e) => {
                        eprintln!("Failed to set position for window {} (PID: {}): {}", 
                                 window.title, window.pid, e);
                    }
                }
            }
        }

        if count == 0 {
            return Err(AppError::NoWindowsModified);
        }

        println!("Successfully positioned {} window(s)", count);
        Ok(())
    }
}

impl Feature for PositionSetFeature {
    fn name(&self) -> &'static str {
        "position_set"
    }
    
    fn description(&self) -> &'static str {
        "Window position setting with layout support"
    }
    
    fn build_cli(&self, command: Command) -> Command {
        command.subcommand(self.build_command())
    }
    
    fn parse_cli(&self, matches: &clap::ArgMatches) -> Option<SubCommand> {
        if let Some(matches) = matches.subcommand_matches("windows/position/set") {
            let (pid, name, title) = Self::extract_filter_args(matches);
            let all = matches.get_flag("all");
            let position = matches.get_one::<String>("position").map(|s| s.to_string());
            let index = matches.get_one::<String>("index").map(|s| s.to_string());
            let layout = matches.get_one::<String>("layout").map(|s| s.to_string());
            let x_start = matches.get_one::<String>("x_start").map(|s| s.to_string());
            let y_start = matches.get_one::<String>("y_start").map(|s| s.to_string());
            let x_step = matches.get_one::<String>("x_step").map(|s| s.to_string());
            let y_step = matches.get_one::<String>("y_step").map(|s| s.to_string());
            
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
            
            Some(SubCommand::WindowsPositionSet { 
                pid, 
                name, 
                title, 
                all,
                position,
                index,
                layout,
                x_start,
                y_start,
                x_step,
                y_step,
                sort_position,
            })
        } else {
            None
        }
    }
    
    fn execute(&self, subcommand: &SubCommand) -> AppResult<()> {
        if let SubCommand::WindowsPositionSet { 
            pid, name, title, all, position, index, layout, 
            x_start, y_start, x_step, y_step, sort_position 
        } = subcommand {
            self.handle_position_set(
                pid.clone(),
                name.clone(), 
                title.clone(),
                *all,
                position.clone(),
                index.clone(),
                layout.clone(),
                x_start.clone(),
                y_start.clone(),
                x_step.clone(),
                y_step.clone(),
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