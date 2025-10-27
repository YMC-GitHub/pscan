// src/features/always_on_top.rs
use clap::{Arg, Command};
use crate::cli::SubCommand;
use super::feature_trait::Feature;
use crate::platform::find_windows;
use crate::error::{AppError, AppResult};

/// 窗口置顶特性
pub struct AlwaysOnTopFeature;

impl AlwaysOnTopFeature {
    pub fn new() -> Self {
        Self
    }
    
    /// 构建子命令
    fn build_command(&self) -> Command {
        Command::new("windows/alwaysontop")
            .about("Set window always on top state")
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
                Arg::new("toggle")
                    .long("toggle")
                    .action(clap::ArgAction::SetTrue)
                    .help("Toggle always on top state (on/off)")
            )
            .arg(
                Arg::new("off")
                    .long("off")
                    .action(clap::ArgAction::SetTrue)
                    .help("Turn off always on top")
                    .conflicts_with("toggle")
            )
    }
    
    /// 统一的字段提取函数
    fn extract_filter_args(matches: &clap::ArgMatches) -> (Option<String>, Option<String>, Option<String>) {
        let pid = matches.get_one::<String>("pid").map(|s| s.to_string());
        let name = matches.get_one::<String>("name").map(|s| s.to_string());
        let title = matches.get_one::<String>("title").map(|s| s.to_string());
        (pid, name, title)
    }
    
    /// 处理置顶命令
    fn handle_always_on_top(
        &self,
        pid_filter: Option<String>,
        name_filter: Option<String>,
        title_filter: Option<String>,
        all: bool,
        toggle: bool,
        off: bool,
    ) -> AppResult<()> {
        // 确定目标状态
        let target_state = if off {
            Some(false)
        } else if toggle {
            None // None 表示切换模式
        } else {
            Some(true) // 默认置顶
        };
        
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
            let result = match target_state {
                Some(state) => {
                    // 直接设置状态
                    window.set_always_on_top(state).map(|_| state)
                }
                None => {
                    // 切换模式：获取当前状态并取反
                    match window.is_always_on_top() {
                        Ok(current_state) => {
                            let new_state = !current_state;
                            window.set_always_on_top(new_state).map(|_| new_state)
                        }
                        Err(e) => Err(e),
                    }
                }
            };

            match result {
                Ok(new_state) => {
                    let state_str = if new_state { "always on top" } else { "normal" };
                    let action_str = if target_state.is_some() { "set" } else { "toggled" };
                    println!("{}: {} (PID: {}) - {}", 
                             action_str, window.title, window.pid, state_str);
                    count += 1;
                }
                Err(e) => {
                    let operation_str = match target_state {
                        Some(true) => "set always on top",
                        Some(false) => "unset always on top", 
                        None => "toggle always on top",
                    };
                    eprintln!("Failed to {} window {} (PID: {}): {}", 
                             operation_str, window.title, window.pid, e);
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

impl Feature for AlwaysOnTopFeature {
    fn name(&self) -> &'static str {
        "always_on_top"
    }
    
    fn description(&self) -> &'static str {
        "Window always on top management"
    }
    
    fn build_cli(&self, command: Command) -> Command {
        command.subcommand(self.build_command())
    }
    
    fn parse_cli(&self, matches: &clap::ArgMatches) -> Option<SubCommand> {
        if let Some(matches) = matches.subcommand_matches("windows/alwaysontop") {
            let (pid, name, title) = Self::extract_filter_args(matches);
            let all = matches.get_flag("all");
            let toggle = matches.get_flag("toggle");
            let off = matches.get_flag("off");
            
            Some(SubCommand::WindowsAlwaysOnTop { 
                pid, 
                name, 
                title, 
                all, 
                toggle,
                off,
            })
        } else {
            None
        }
    }
    
    fn execute(&self, subcommand: &SubCommand) -> AppResult<()> {
        if let SubCommand::WindowsAlwaysOnTop { pid, name, title, all, toggle, off } = subcommand {
            self.handle_always_on_top(
                pid.clone(),
                name.clone(), 
                title.clone(),
                *all,
                *toggle,
                *off,
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