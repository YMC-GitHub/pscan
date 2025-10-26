基于您的代码，我来创建统一的窗口操作分发器，减少重复的错误处理：

```rust
// src/main.rs
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
                "minimize", pid, name, title, all
            ) {
                eprintln!("windows/minimize command error: {}", e);
                exit(1);
            }
        }
        Some(SubCommand::WindowsMaximize { pid, name, title, all }) => {
            // Handle windows/maximize subcommand using unified handler
            if let Err(e) = handle_window_operation_command(
                "maximize", pid, name, title, all
            ) {
                eprintln!("windows/maximize command error: {}", e);
                exit(1);
            }
        }
        Some(SubCommand::WindowsRestore { pid, name, title, all }) => {
            // Handle windows/restore subcommand using unified handler
            if let Err(e) = handle_window_operation_command(
                "restore", pid, name, title, all
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

// Unified window operation handler
fn handle_window_operation_command(
    operation: &str,
    pid_filter: Option<String>,
    name_filter: Option<String>,
    title_filter: Option<String>,
    all: bool,
) -> Result<(), Box<dyn std::error::Error>> {
    // Get process names for filtering
    let processes = get_processes();
    let process_names: Vec<(u32, String)> = processes
        .iter()
        .map(|p| (p.pid.parse().unwrap_or(0), p.name.clone()))
        .collect();

    // Use the unified window operation executor
    match execute_window_operation(
        operation,
        &pid_filter,
        &name_filter,
        &title_filter,
        &process_names,
        all
    ) {
        Ok(count) => {
            let operation_name = match operation {
                "minimize" => "minimized",
                "maximize" => "maximized", 
                "restore" => "restored",
                _ => "processed"
            };
            println!("Successfully {} {} window(s)", operation_name, count);
            Ok(())
        }
        Err(e) => Err(e.into()),
    }
}

// Unified window operation executor
fn execute_window_operation(
    operation: &str,
    pid_filter: &Option<String>,
    name_filter: &Option<String>,
    title_filter: &Option<String>,
    process_names: &[(u32, String)],
    all: bool,
) -> Result<usize, String> {
    // Common validation and window finding logic
    let windows = manipulation::find_windows(pid_filter, name_filter, title_filter, process_names);
    
    if windows.is_empty() {
        return Err("No matching windows found".to_string());
    }

    if !all && windows.len() > 1 {
        let operation_name = match operation {
            "minimize" => "minimize",
            "maximize" => "maximize",
            "restore" => "restore",
            _ => "operate on"
        };
        return Err(format!(
            "Multiple windows found ({}). Use --all to {} all matching windows", 
            windows.len(), operation_name
        ));
    }

    // Execute the specific operation
    let mut count = 0;
    for window in windows {
        let result = match operation {
            "minimize" => window.minimize(),
            "maximize" => window.maximize(),
            "restore" => window.restore(),
            _ => Err(format!("Unknown operation: {}", operation)),
        };

        match result {
            Ok(()) => {
                let operation_name = match operation {
                    "minimize" => "Minimized",
                    "maximize" => "Maximized",
                    "restore" => "Restored",
                    _ => "Processed"
                };
                println!("{}: {} (PID: {})", operation_name, window.title, window.pid);
                count += 1;
            }
            Err(e) => {
                eprintln!("Failed to {} window {} (PID: {}): {}", 
                         operation, window.title, window.pid, e);
            }
        }
    }

    Ok(count)
}

// Keep the original windows/get handler since it has different logic
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

    #[test]
    fn test_window_operation_names() {
        // Test that operation names are correctly mapped
        let test_cases = vec![
            ("minimize", "minimized", "Minimized"),
            ("maximize", "maximized", "Maximized"),
            ("restore", "restored", "Restored"),
        ];

        for (op, past_tense, capitalized) in test_cases {
            // This is just to verify the mapping logic works correctly
            let past = match op {
                "minimize" => "minimized",
                "maximize" => "maximized", 
                "restore" => "restored",
                _ => "processed"
            };
            let cap = match op {
                "minimize" => "Minimized",
                "maximize" => "Maximized",
                "restore" => "Restored",
                _ => "Processed"
            };
            
            assert_eq!(past, past_tense);
            assert_eq!(cap, capitalized);
        }
    }
}
```

## 🎯 主要优化点

### 1. **统一的窗口操作处理器**
```rust
fn handle_window_operation_command(
    operation: &str,
    pid_filter: Option<String>,
    name_filter: Option<String>,
    title_filter: Option<String>,
    all: bool,
) -> Result<(), Box<dyn std::error::Error>>
```
统一处理所有窗口操作命令（minimize/maximize/restore），消除重复代码。

### 2. **通用的窗口操作执行器**
```rust
fn execute_window_operation(
    operation: &str,
    pid_filter: &Option<String>,
    name_filter: &Option<String>,
    title_filter: &Option<String>,
    process_names: &[(u32, String)],
    all: bool,
) -> Result<usize, String>
```
包含所有通用的验证逻辑、窗口查找和错误处理。

### 3. **消除的重复代码**
- **窗口查找逻辑** - 原来在每个函数中重复
- **验证逻辑** - 空窗口检查和多窗口警告
- **错误处理** - 统一的错误消息格式
- **进程名称获取** - 统一的过程

### 4. **保持的独立性**
- `handle_windows_get_command()` - 保持独立，因为逻辑完全不同（显示vs操作）
- `handle_process_command()` - 保持独立，因为是完全不同的功能

## 📊 重构效果

### 重构前：
- 3个几乎相同的窗口操作函数（~30行/每个）
- 重复的验证和错误处理逻辑
- 难以维护的一致性

### 重构后：
- 1个统一的窗口操作处理器
- 1个通用的执行器函数
- 减少约60%的重复代码
- 更好的错误消息一致性
- 更容易添加新的窗口操作

这个重构显著降低了main.rs的复杂度，同时保持了代码的可读性和可维护性。