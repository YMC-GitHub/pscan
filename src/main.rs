// src/main.rs
mod types;
mod cli;
mod process;
mod window;
mod output;
mod platform;
mod sorting;
mod utils;
mod features;  // 新增特性模块
mod error;     // 新增错误处理模块

use std::process::exit;
use output::{OutputFormat, display_processes};
use cli::{parse_args, SubCommand};
use process::{get_processes, filter_processes};
use features::{create_default_manager, get_enabled_features};  // 新增
use error::{AppError, AppResult};  // 新增

fn main() {
    if let Err(e) = run() {
        eprintln!("Error: {}", e);
        
        // 根据错误类型决定退出码
        let exit_code = match e {
            AppError::NoMatchingWindows => 2,
            AppError::MultipleWindows(_) => 3,
            AppError::InvalidParameter(_) => 4,
            AppError::FeatureNotSupported(_) => 5,
            _ => 1,
        };
        
        exit(exit_code);
    }
}

fn run() -> AppResult<()> {
    let config = parse_args();
    let feature_manager = create_default_manager();  // 创建特性管理器

    // 显示启用的特性（调试信息）
    if config.verbose {
        let enabled_features = get_enabled_features();
        if !enabled_features.is_empty() {
            println!("Enabled features: {:?}", enabled_features);
        }
        
        let runtime_features: Vec<&str> = feature_manager.get_features()
            .iter()
            .map(|f| f.name())
            .collect();
        if !runtime_features.is_empty() {
            println!("Runtime available features: {:?}", runtime_features);
        }
    }

    match config.subcommand {
        // 所有子命令现在都由特性管理器处理
        Some(subcommand) => {
            feature_manager.execute(&subcommand)?;
        }
        None => {
            // Handle normal process listing
            handle_process_command(config)?;
        }
    }
    
    Ok(())
}

// 窗口操作类型枚举 - 提供类型安全
#[derive(Debug, Clone, Copy)]
enum WindowOperation {
    Minimize,
    Maximize,
    Restore,
}

impl WindowOperation {
    // 获取操作名称（动词形式）
    fn as_str(&self) -> &'static str {
        match self {
            WindowOperation::Minimize => "minimize",
            WindowOperation::Maximize => "maximize",
            WindowOperation::Restore => "restore",
        }
    }
    
    // 获取过去式形式（用于成功消息）
    fn past_tense(&self) -> &'static str {
        match self {
            WindowOperation::Minimize => "minimized",
            WindowOperation::Maximize => "maximized",
            WindowOperation::Restore => "restored",
        }
    }
    
    // 获取首字母大写形式（用于操作日志）
    fn capitalized(&self) -> &'static str {
        match self {
            WindowOperation::Minimize => "Minimized",
            WindowOperation::Maximize => "Maximized",
            WindowOperation::Restore => "Restored",
        }
    }
}

// 统一的窗口操作处理函数
fn handle_window_operation_command(
    pid_filter: Option<String>,
    name_filter: Option<String>,
    title_filter: Option<String>,
    all: bool,
    operation: WindowOperation,
) -> AppResult<()> {
    // Get process names for filtering
    let processes = get_processes();
    let process_names: Vec<(u32, String)> = processes
        .iter()
        .map(|p| (p.pid.parse().unwrap_or(0), p.name.clone()))
        .collect();

    // 使用统一的执行器
    let count = execute_window_operation(
        operation,
        &pid_filter,
        &name_filter,
        &title_filter,
        &process_names,
        all
    )?;
    
    println!("Successfully {} {} window(s)", operation.past_tense(), count);
    Ok(())
}

// 统一的窗口操作执行器 - 消除重复逻辑
fn execute_window_operation(
    operation: WindowOperation,
    pid_filter: &Option<String>,
    name_filter: &Option<String>,
    title_filter: &Option<String>,
    process_names: &[(u32, String)],
    all: bool,
) -> AppResult<usize> {
    // 使用平台抽象层查找匹配的窗口
    let windows = crate::platform::find_windows(pid_filter, name_filter, title_filter, process_names);
    
    // 验证窗口数量
    if windows.is_empty() {
        return Err(AppError::NoMatchingWindows);
    }

    if !all && windows.len() > 1 {
        return Err(AppError::MultipleWindows(windows.len()));
    }

    // 执行操作
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

    Ok(count)
}

// 进程列表处理函数（保持独立）
fn handle_process_command(config: cli::CliConfig) -> AppResult<()> {
    // Get process list
    let processes = get_processes();

    // Filter processes
    let filtered_processes = filter_processes(
        &processes,
        &config.pid_filter,
        &config.name_filter,
        &config.title_filter,
        config.has_window_filter,
        config.no_window_filter,
    );

    // Display results
    if filtered_processes.is_empty() {
        return Err(AppError::NoMatchingWindows);
    }

    display_processes(&filtered_processes, config.format, config.verbose)
}

#[cfg(test)]
mod tests {
    use super::*;
    use output::truncate_string;
    use sorting::{SortOrder, PositionSort};
    use types::WindowRect;

    #[test]
    fn test_truncate_string() {
        assert_eq!(truncate_string("hello", 5), "hello");
        assert_eq!(truncate_string("hello world", 8), "hello...");
        assert_eq!(truncate_string("hi", 10), "hi");
    }

    #[test]
    fn test_window_operation_enum() {
        // Test operation name mappings
        let minimize = WindowOperation::Minimize;
        let maximize = WindowOperation::Maximize;
        let restore = WindowOperation::Restore;

        assert_eq!(minimize.as_str(), "minimize");
        assert_eq!(maximize.as_str(), "maximize");
        assert_eq!(restore.as_str(), "restore");

        assert_eq!(minimize.past_tense(), "minimized");
        assert_eq!(maximize.past_tense(), "maximized");
        assert_eq!(restore.past_tense(), "restored");

        assert_eq!(minimize.capitalized(), "Minimized");
        assert_eq!(maximize.capitalized(), "Maximized");
        assert_eq!(restore.capitalized(), "Restored");
    }

    #[test]
    fn test_window_operation_clone() {
        // Test that the enum can be cloned (needed for function parameters)
        let op1 = WindowOperation::Minimize;
        let op2 = op1;
        let op3 = op1.clone();
        
        assert_eq!(op1.as_str(), op2.as_str());
        assert_eq!(op1.as_str(), op3.as_str());
    }

    #[test]
    fn test_window_operation_debug() {
        // Test Debug trait implementation
        let minimize = WindowOperation::Minimize;
        let maximize = WindowOperation::Maximize;
        let restore = WindowOperation::Restore;

        // This should compile and run without panicking
        format!("{:?}", minimize);
        format!("{:?}", maximize);
        format!("{:?}", restore);
    }

    #[test]
    fn test_window_operation_copy() {
        // Test that the enum implements Copy trait
        let op1 = WindowOperation::Minimize;
        let op2 = op1; // This should work if Copy is implemented
        
        // Both should be usable
        assert_eq!(op1.as_str(), op2.as_str());
    }
}