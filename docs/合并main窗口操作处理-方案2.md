我来创建一个统一的窗口操作分发器，减少重复的错误处理代码。以下是重构后的 `src/main.rs`：

```rust
mod types;
mod cli;
mod process;
mod window;
mod output;

use std::process::exit;
use output::{OutputFormat, display_processes, display_windows};
use cli::{parse_args, SubCommand};
use process::{get_processes, filter_processes};
use window::{get_all_windows_with_size, manipulation};
use types::WindowInfo;

fn main() {
    let config = parse_args();

    match config.subcommand {
        Some(SubCommand::WindowsGet { pid, name, title, format }) => {
            // Handle windows/get subcommand
            if let Err(e) = handle_windows_get_command(pid, name, title, format) {
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
        None => {
            // Handle normal process listing
            handle_process_command(config);
        }
    }
}

// 窗口操作类型枚举
enum WindowOperation {
    Minimize,
    Maximize,
    Restore,
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

    // 根据操作类型调用相应的函数
    let result = match operation {
        WindowOperation::Minimize => {
            manipulation::minimize_windows(&pid_filter, &name_filter, &title_filter, &process_names, all)
        }
        WindowOperation::Maximize => {
            manipulation::maximize_windows(&pid_filter, &name_filter, &title_filter, &process_names, all)
        }
        WindowOperation::Restore => {
            manipulation::restore_windows(&pid_filter, &name_filter, &title_filter, &process_names, all)
        }
    };

    match result {
        Ok(count) => {
            let operation_name = match operation {
                WindowOperation::Minimize => "minimized",
                WindowOperation::Maximize => "maximized",
                WindowOperation::Restore => "restored",
            };
            println!("Successfully {} {} window(s)", operation_name, count);
            Ok(())
        }
        Err(e) => Err(e.into()),
    }
}

fn handle_windows_get_command(
    pid_filter: Option<String>,
    name_filter: Option<String>,
    title_filter: Option<String>,
    format: OutputFormat,
) -> Result<(), Box<dyn std::error::Error>> {
    // Get all windows with size information
    let windows = get_all_windows_with_size();
    
    // Get process names for display
    let processes = get_processes();
    let process_names: Vec<(u32, String)> = processes
        .iter()
        .map(|p| (p.pid.parse().unwrap_or(0), p.name.clone()))
        .collect();
    
    // Filter windows
    let filtered_windows: Vec<&WindowInfo> = windows
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
                    .find(|(pid, _)| *pid == window.pid)
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
        .collect();

    if filtered_windows.is_empty() {
        eprintln!("No matching windows found");
        exit(1);
    }

    // Convert &Vec<&WindowInfo> to &[WindowInfo] by dereferencing
    let windows_slice: Vec<WindowInfo> = filtered_windows.iter().map(|&w| (*w).clone()).collect();
    display_windows(&windows_slice, &process_names, format)
}

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

    #[test]
    fn test_truncate_string() {
        assert_eq!(truncate_string("hello", 5), "hello");
        assert_eq!(truncate_string("hello world", 8), "hello...");
        assert_eq!(truncate_string("hi", 10), "hi");
    }
}
```

**主要改进：**

1. **创建了 `WindowOperation` 枚举**：
   - 定义了三种窗口操作类型：Minimize、Maximize、Restore
   - 使操作类型更加类型安全

2. **创建了统一的 `handle_window_operation_command` 函数**：
   - 合并了原来三个独立的处理函数（minimize、maximize、restore）
   - 统一的错误处理逻辑
   - 统一的进程信息获取逻辑
   - 统一的结果输出格式

3. **减少了重复代码**：
   - 原来三个函数中重复的进程信息获取逻辑现在只在一个地方
   - 错误处理和结果输出逻辑统一
   - 参数处理逻辑统一

4. **保持了原有的接口兼容性**：
   - 主函数中的调用方式基本不变
   - 只是内部实现被统一处理

**优点：**

- **代码复用**：避免了三个几乎相同的函数
- **维护性**：修改窗口操作逻辑时只需修改一个地方
- **一致性**：所有窗口操作的处理方式完全一致
- **可扩展性**：添加新的窗口操作类型时只需扩展枚举和匹配分支

这样的重构使得代码更加简洁，减少了重复，提高了可维护性。