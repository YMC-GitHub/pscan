// src/features/transparency.rs
use clap::{Arg, Command};
use crate::cli::SubCommand;
use super::feature_trait::Feature;
use crate::platform::find_windows;

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
        level: u8,
        reset: bool,
    ) -> Result<(), Box<dyn std::error::Error>> {
        // 确定透明度级别
        let target_level = if reset { 100 } else { level };
        
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
            return Err("No matching windows found".to_string().into());
        }

        if !all && windows.len() > 1 {
            return Err(format!(
                "Multiple windows found ({}). Use --all to modify all matching windows", 
                windows.len()
            ).into());
        }

        let mut count = 0;
        for window in windows {
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
            return Err("No windows were modified".to_string().into());
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
            let level = *matches.get_one::<u8>("level").unwrap_or(&100);
            let reset = matches.get_flag("reset");
            
            Some(SubCommand::WindowsTransparency { 
                pid, 
                name, 
                title, 
                all, 
                level,
                reset,
            })
        } else {
            None
        }
    }
    
    fn execute(&self, subcommand: &SubCommand) -> Result<(), Box<dyn std::error::Error>> {
        if let SubCommand::WindowsTransparency { pid, name, title, all, level, reset } = subcommand {
            self.handle_transparency(
                pid.clone(),
                name.clone(), 
                title.clone(),
                *all,
                *level,
                *reset,
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