// src/main.rs
mod types;
mod cli;
mod process;
mod window;
mod output;
mod platform;
mod sorting;
mod utils;  // 新增工具模块

use std::process::exit;
use output::{OutputFormat, display_processes, display_windows};
use cli::{parse_args, SubCommand};
use sorting::{SortOrder, PositionSort};
use process::{get_processes, filter_processes};
use window::{get_all_windows_with_size, find_windows};
use types::WindowInfo;
use utils::{parse_indices, validate_position_parameters, calculate_positions};  // 使用工具函数

fn main() {
    let config = parse_args();

    match config.subcommand {
        Some(SubCommand::WindowsGet { pid, name, title, format, sort_pid, sort_position }) => {
            // Handle windows/get subcommand
            if let Err(e) = handle_windows_get_command(pid, name, title, format, sort_pid, sort_position) {
                eprintln!("windows/get command error: {}", e);
                exit(1);
            }
        }
        Some(SubCommand::WindowsMinimize { pid, name, title, all }) => {
            // Handle windows/minimize subcommand using unified handler
            if let Err(e) = handle_window_operation_command(
                pid, name, title, all, 
                WindowOperation::Minimize
            ) {
                eprintln!("windows/minimize command error: {}", e);
                exit(1);
            }
        }
        Some(SubCommand::WindowsMaximize { pid, name, title, all }) => {
            // Handle windows/maximize subcommand using unified handler
            if let Err(e) = handle_window_operation_command(
                pid, name, title, all, 
                WindowOperation::Maximize
            ) {
                eprintln!("windows/maximize command error: {}", e);
                exit(1);
            }
        }
        Some(SubCommand::WindowsRestore { pid, name, title, all }) => {
            // Handle windows/restore subcommand using unified handler
            if let Err(e) = handle_window_operation_command(
                pid, name, title, all, 
                WindowOperation::Restore
            ) {
                eprintln!("windows/restore command error: {}", e);
                exit(1);
            }
        }
        Some(SubCommand::WindowsPositionSet { 
            pid, name, title, all, position, index, layout, 
            x_start, y_start, x_step, y_step, sort_position 
        }) => {
            if let Err(e) = handle_windows_position_set_command(
                pid, name, title, all, position, index, layout,
                x_start, y_start, x_step, y_step, sort_position
            ) {
                eprintln!("windows/position/set command error: {}", e);
                exit(1);
            }
        }
        None => {
            // Handle normal process listing
            handle_process_command(config);
        }
    }
}

// 窗口操作类型枚举 - 提供类型安全
#[derive(Debug, Clone, Copy)]
enum WindowOperation {
    Minimize,
    Maximize,
    Restore,
    #[allow(dead_code)] 
    SetPosition,
}

impl WindowOperation {
    // 获取操作名称（动词形式）
    fn as_str(&self) -> &'static str {
        match self {
            WindowOperation::Minimize => "minimize",
            WindowOperation::Maximize => "maximize",
            WindowOperation::Restore => "restore",
            WindowOperation::SetPosition => "set position",
        }
    }
    
    // 获取过去式形式（用于成功消息）
    fn past_tense(&self) -> &'static str {
        match self {
            WindowOperation::Minimize => "minimized",
            WindowOperation::Maximize => "maximized",
            WindowOperation::Restore => "restored",
            WindowOperation::SetPosition => "position set",
        }
    }
    
    // 获取首字母大写形式（用于操作日志）
    fn capitalized(&self) -> &'static str {
        match self {
            WindowOperation::Minimize => "Minimized",
            WindowOperation::Maximize => "Maximized",
            WindowOperation::Restore => "Restored",
            WindowOperation::SetPosition => "Position set",
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
) -> Result<(), Box<dyn std::error::Error>> {
    // Get process names for filtering
    let processes = get_processes();
    let process_names: Vec<(u32, String)> = processes
        .iter()
        .map(|p| (p.pid.parse().unwrap_or(0), p.name.clone()))
        .collect();

    // 使用统一的执行器
    match execute_window_operation(
        operation,
        &pid_filter,
        &name_filter,
        &title_filter,
        &process_names,
        all
    ) {
        Ok(count) => {
            println!("Successfully {} {} window(s)", operation.past_tense(), count);
            Ok(())
        }
        Err(e) => Err(e.into()),
    }
}

// 统一的窗口操作执行器 - 消除重复逻辑
fn execute_window_operation(
    operation: WindowOperation,
    pid_filter: &Option<String>,
    name_filter: &Option<String>,
    title_filter: &Option<String>,
    process_names: &[(u32, String)],
    all: bool,
) -> Result<usize, String> {
    // 使用平台抽象层查找匹配的窗口
    let windows = find_windows(pid_filter, name_filter, title_filter, process_names);
    
    // 验证窗口数量
    if windows.is_empty() {
        return Err("No matching windows found".to_string());
    }

    if !all && windows.len() > 1 {
        return Err(format!(
            "Multiple windows found ({}). Use --all to {} all matching windows", 
            windows.len(), operation.as_str()
        ));
    }

    // 执行操作
    let mut count = 0;
    for window in windows {
        let result = match operation {
            WindowOperation::Minimize => window.minimize(),
            WindowOperation::Maximize => window.maximize(),
            WindowOperation::Restore => window.restore(),
            WindowOperation::SetPosition => {
                // 对于位置设置，需要额外的参数，这里不处理
                continue;
            }
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

// 更新 windows/get 处理函数
fn handle_windows_get_command(
    pid_filter: Option<String>,
    name_filter: Option<String>,
    title_filter: Option<String>,
    format: OutputFormat,
    sort_pid: SortOrder,
    sort_position: PositionSort,
) -> Result<(), Box<dyn std::error::Error>> {
    // 使用平台抽象层获取所有窗口及其尺寸信息
    let windows = get_all_windows_with_size();
    
    // Get process names for display
    let processes = get_processes();
    let process_names: Vec<(u32, String)> = processes
        .iter()
        .map(|p| (p.pid.parse().unwrap_or(0), p.name.clone()))
        .collect();
    
    // Filter windows
    let mut filtered_windows: Vec<WindowInfo> = windows
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
        eprintln!("No matching windows found");
        exit(1);
    }

    // 打印排序前的 PID 列表（调试用）
    if std::env::var("DEBUG_SORT").is_ok() {
        println!("Before sorting:");
        for window in &filtered_windows {
            println!("  PID: {}, Title: {}", window.pid, window.title);
        }
    }

    // 应用排序 - 使用 sorting 模块的函数
    sorting::apply_window_sorting(&mut filtered_windows, &sort_pid, &sort_position);

    // 打印排序后的 PID 列表（调试用）
    if std::env::var("DEBUG_SORT").is_ok() {
        println!("After sorting:");
        for window in &filtered_windows {
            println!("  PID: {}, Title: {}", window.pid, window.title);
        }
    }

    // Convert to slice for display
    display_windows(&filtered_windows, &process_names, format)
}

// 添加新的位置设置处理函数
fn handle_windows_position_set_command(
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
) -> Result<(), Box<dyn std::error::Error>> {
    // 获取进程名称用于过滤
    let processes = get_processes();
    let process_names: Vec<(u32, String)> = processes
        .iter()
        .map(|p| (p.pid.parse().unwrap_or(0), p.name.clone()))
        .collect();

    // 使用平台抽象层查找匹配的窗口
    let mut windows = find_windows(&pid_filter, &name_filter, &title_filter, &process_names);
    
    // 验证窗口数量
    if windows.is_empty() {
        return Err("No matching windows found".to_string().into());  // 修复错误类型
    }

    // 应用排序 - 使用 sorting 模块的函数
    sorting::apply_window_handle_sorting(&mut windows, &SortOrder::None, &sort_position);

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
        return Err("No windows were positioned".to_string().into());  // 修复错误类型
    }

    println!("Successfully positioned {} window(s)", count);
    Ok(())
}

// 进程列表处理函数（保持独立）
fn handle_process_command(config: cli::CliConfig) {
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
        eprintln!("No matching processes found");
        exit(1);
    }

    if let Err(e) = display_processes(&filtered_processes, config.format, config.verbose) {
        eprintln!("Output error: {}", e);
        exit(1);
    }
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
        let set_position = WindowOperation::SetPosition;

        assert_eq!(minimize.as_str(), "minimize");
        assert_eq!(maximize.as_str(), "maximize");
        assert_eq!(restore.as_str(), "restore");
        assert_eq!(set_position.as_str(), "set position");

        assert_eq!(minimize.past_tense(), "minimized");
        assert_eq!(maximize.past_tense(), "maximized");
        assert_eq!(restore.past_tense(), "restored");
        assert_eq!(set_position.past_tense(), "position set");

        assert_eq!(minimize.capitalized(), "Minimized");
        assert_eq!(maximize.capitalized(), "Maximized");
        assert_eq!(restore.capitalized(), "Restored");
        assert_eq!(set_position.capitalized(), "Position set");
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
        let set_position = WindowOperation::SetPosition;

        // This should compile and run without panicking
        format!("{:?}", minimize);
        format!("{:?}", maximize);
        format!("{:?}", restore);
        format!("{:?}", set_position);
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