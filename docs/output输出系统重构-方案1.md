基于您的代码，我来重构输出系统，使用策略模式处理不同输出格式，并提取通用的显示逻辑：

```rust
// src/output.rs
use crate::types::{ProcessInfo, ProcessOutput, WindowInfo, WindowOutput};

#[derive(Debug, Clone, clap::ValueEnum)]
pub enum OutputFormat {
    Table,
    Json,
    Yaml,
    Csv,
    Simple,
    Detailed,
}

// 输出策略 trait
trait OutputStrategy<T> {
    fn display(&self, data: &[T], process_names: Option<&[(u32, String)]>) -> Result<(), Box<dyn std::error::Error>>;
}

// 进程输出策略
mod process_strategies {
    use super::*;
    use csv::Writer;
    use serde_json;
    use serde_yaml;

    pub struct TableStrategy {
        pub verbose: bool,
    }

    impl OutputStrategy<ProcessInfo> for TableStrategy {
        fn display(&self, processes: &[ProcessInfo], _process_names: Option<&[(u32, String)]>) -> Result<(), Box<dyn std::error::Error>> {
            println!("Found {} matching processes:", processes.len());
            
            if self.verbose {
                println!("{:<8} {:<20} {:<30} {:<12} {}", 
                         "PID", "Name", "Title", "Memory", "Window");
            } else {
                println!("{:<8} {:<20} {:<30} {}", 
                         "PID", "Name", "Title", "Memory");
            }

            for process in processes {
                let memory_mb = process.memory_usage as f64 / 1024.0 / 1024.0;
                
                if self.verbose {
                    println!(
                        "{:<8} {:<20} {:<30} {:<11.2} MB {}",
                        process.pid,
                        truncate_string(&process.name, 18),
                        truncate_string(&process.title, 28),
                        memory_mb,
                        if process.has_window { "Yes" } else { "No" }
                    );
                } else {
                    println!(
                        "{:<8} {:<20} {:<30} {:.2} MB",
                        process.pid,
                        truncate_string(&process.name, 18),
                        truncate_string(&process.title, 28),
                        memory_mb
                    );
                }

                if self.verbose {
                    println!("    PID: {}", process.pid);
                    println!("    Name: {}", process.name);
                    println!("    Title: {}", process.title);
                    println!("    Memory: {:.2} MB", memory_mb);
                    println!("    Has Window: {}", if process.has_window { "Yes" } else { "No" });
                    println!("    {}", "-".repeat(50));
                }
            }
            
            Ok(())
        }
    }

    pub struct JsonStrategy;

    impl OutputStrategy<ProcessInfo> for JsonStrategy {
        fn display(&self, processes: &[ProcessInfo], _process_names: Option<&[(u32, String)]>) -> Result<(), Box<dyn std::error::Error>> {
            let output: Vec<ProcessOutput> = processes.iter().map(|p| ProcessOutput::from(p)).collect();
            let json = serde_json::to_string_pretty(&output)?;
            println!("{}", json);
            Ok(())
        }
    }

    pub struct YamlStrategy;

    impl OutputStrategy<ProcessInfo> for YamlStrategy {
        fn display(&self, processes: &[ProcessInfo], _process_names: Option<&[(u32, String)]>) -> Result<(), Box<dyn std::error::Error>> {
            let output: Vec<ProcessOutput> = processes.iter().map(|p| ProcessOutput::from(p)).collect();
            let yaml = serde_yaml::to_string(&output)?;
            println!("{}", yaml);
            Ok(())
        }
    }

    pub struct CsvStrategy;

    impl OutputStrategy<ProcessInfo> for CsvStrategy {
        fn display(&self, processes: &[ProcessInfo], _process_names: Option<&[(u32, String)]>) -> Result<(), Box<dyn std::error::Error>> {
            let mut wtr = Writer::from_writer(std::io::stdout());
            
            wtr.write_record(&["PID", "Name", "Title", "MemoryUsage", "MemoryUsageMB", "HasWindow"])?;
            
            for process in processes {
                let output = ProcessOutput::from(process);
                wtr.write_record(&[
                    &output.pid,
                    &output.name,
                    &output.title,
                    &output.memory_usage.to_string(),
                    &format!("{:.2}", output.memory_usage_mb),
                    &output.has_window.to_string(),
                ])?;
            }
            
            wtr.flush()?;
            Ok(())
        }
    }

    pub struct SimpleStrategy;

    impl OutputStrategy<ProcessInfo> for SimpleStrategy {
        fn display(&self, processes: &[ProcessInfo], _process_names: Option<&[(u32, String)]>) -> Result<(), Box<dyn std::error::Error>> {
            for process in processes {
                let memory_mb = process.memory_usage as f64 / 1024.0 / 1024.0;
                println!(
                    "{}: {} ({} MB) - {}",
                    process.pid,
                    process.name,
                    format!("{:.1}", memory_mb),
                    if process.has_window { "Has Window" } else { "No Window" }
                );
            }
            Ok(())
        }
    }

    pub struct DetailedStrategy;

    impl OutputStrategy<ProcessInfo> for DetailedStrategy {
        fn display(&self, processes: &[ProcessInfo], _process_names: Option<&[(u32, String)]>) -> Result<(), Box<dyn std::error::Error>> {
            for (i, process) in processes.iter().enumerate() {
                let memory_mb = process.memory_usage as f64 / 1024.0 / 1024.0;
                println!("Process #{}:", i + 1);
                println!("  PID:          {}", process.pid);
                println!("  Name:         {}", process.name);
                println!("  Title:        {}", process.title);
                println!("  Memory:       {:.2} MB", memory_mb);
                println!("  Raw Memory:   {} bytes", process.memory_usage);
                println!("  Has Window:   {}", if process.has_window { "Yes" } else { "No" });
                println!();
            }
            Ok(())
        }
    }
}

// 窗口输出策略
mod window_strategies {
    use super::*;
    use csv::Writer;
    use serde_json;
    use serde_yaml;

    pub struct TableStrategy;

    impl OutputStrategy<WindowInfo> for TableStrategy {
        fn display(&self, windows: &[WindowInfo], process_names: Option<&[(u32, String)]>) -> Result<(), Box<dyn std::error::Error>> {
            let process_names = process_names.expect("Process names required for window table display");
            
            println!("Found {} windows:", windows.len());
            println!("{:<8} {:<20} {:<30} {:<15} {:<12}", 
                     "PID", "Name", "Title", "Size", "Position");
            
            for window in windows {
                let process_name = process_names
                    .iter()
                    .find(|(pid, _)| *pid == window.pid)
                    .map(|(_, name)| name.as_str())
                    .unwrap_or("Unknown");
                
                println!(
                    "{:<8} {:<20} {:<30} {:<8}x{:<6} +{}+{}",
                    window.pid,
                    truncate_string(process_name, 18),
                    truncate_string(&window.title, 28),
                    window.rect.width,
                    window.rect.height,
                    window.rect.x,
                    window.rect.y
                );
            }
            
            Ok(())
        }
    }

    pub struct JsonStrategy;

    impl OutputStrategy<WindowInfo> for JsonStrategy {
        fn display(&self, windows: &[WindowInfo], process_names: Option<&[(u32, String)]>) -> Result<(), Box<dyn std::error::Error>> {
            let process_names = process_names.expect("Process names required for window JSON display");
            
            let output: Vec<WindowOutput> = windows.iter()
                .map(|window| {
                    let mut output = WindowOutput::from(window);
                    output.name = process_names
                        .iter()
                        .find(|(pid, _)| *pid == window.pid)
                        .map(|(_, name)| name.clone())
                        .unwrap_or_else(|| "Unknown".to_string());
                    output
                })
                .collect();
            let json = serde_json::to_string_pretty(&output)?;
            println!("{}", json);
            Ok(())
        }
    }

    pub struct YamlStrategy;

    impl OutputStrategy<WindowInfo> for YamlStrategy {
        fn display(&self, windows: &[WindowInfo], process_names: Option<&[(u32, String)]>) -> Result<(), Box<dyn std::error::Error>> {
            let process_names = process_names.expect("Process names required for window YAML display");
            
            let output: Vec<WindowOutput> = windows.iter()
                .map(|window| {
                    let mut output = WindowOutput::from(window);
                    output.name = process_names
                        .iter()
                        .find(|(pid, _)| *pid == window.pid)
                        .map(|(_, name)| name.clone())
                        .unwrap_or_else(|| "Unknown".to_string());
                    output
                })
                .collect();
            let yaml = serde_yaml::to_string(&output)?;
            println!("{}", yaml);
            Ok(())
        }
    }

    pub struct CsvStrategy;

    impl OutputStrategy<WindowInfo> for CsvStrategy {
        fn display(&self, windows: &[WindowInfo], process_names: Option<&[(u32, String)]>) -> Result<(), Box<dyn std::error::Error>> {
            let process_names = process_names.expect("Process names required for window CSV display");
            
            let mut wtr = Writer::from_writer(std::io::stdout());
            
            wtr.write_record(&["PID", "Name", "Title", "X", "Y", "Width", "Height", "Dimensions"])?;
            
            for window in windows {
                let process_name = process_names
                    .iter()
                    .find(|(pid, _)| *pid == window.pid)
                    .map(|(_, name)| name.as_str())
                    .unwrap_or("Unknown");
                    
                wtr.write_record(&[
                    &window.pid.to_string(),
                    process_name,
                    &window.title,
                    &window.rect.x.to_string(),
                    &window.rect.y.to_string(),
                    &window.rect.width.to_string(),
                    &window.rect.height.to_string(),
                    &window.rect.to_string(),
                ])?;
            }
            
            wtr.flush()?;
            Ok(())
        }
    }

    pub struct SimpleStrategy;

    impl OutputStrategy<WindowInfo> for SimpleStrategy {
        fn display(&self, windows: &[WindowInfo], process_names: Option<&[(u32, String)]>) -> Result<(), Box<dyn std::error::Error>> {
            let process_names = process_names.expect("Process names required for window simple display");
            
            for window in windows {
                let process_name = process_names
                    .iter()
                    .find(|(pid, _)| *pid == window.pid)
                    .map(|(_, name)| name.as_str())
                    .unwrap_or("Unknown");
                    
                println!(
                    "{}: {} - {} ({}x{} at +{}+{})",
                    window.pid,
                    process_name,
                    window.title,
                    window.rect.width,
                    window.rect.height,
                    window.rect.x,
                    window.rect.y
                );
            }
            Ok(())
        }
    }

    pub struct DetailedStrategy;

    impl OutputStrategy<WindowInfo> for DetailedStrategy {
        fn display(&self, windows: &[WindowInfo], process_names: Option<&[(u32, String)]>) -> Result<(), Box<dyn std::error::Error>> {
            let process_names = process_names.expect("Process names required for window detailed display");
            
            for (i, window) in windows.iter().enumerate() {
                let process_name = process_names
                    .iter()
                    .find(|(pid, _)| *pid == window.pid)
                    .map(|(_, name)| name.as_str())
                    .unwrap_or("Unknown");
                    
                println!("Window #{}:", i + 1);
                println!("  PID:        {}", window.pid);
                println!("  Name:       {}", process_name);
                println!("  Title:      {}", window.title);
                println!("  Size:       {}x{}", window.rect.width, window.rect.height);
                println!("  Position:   +{}+{}", window.rect.x, window.rect.y);
                println!("  Dimensions: {}", window.rect.to_string());
                println!();
            }
            Ok(())
        }
    }
}

// 输出上下文 - 统一的输出接口
pub struct OutputContext<T> {
    strategy: Box<dyn OutputStrategy<T>>,
    process_names: Option<Vec<(u32, String)>>,
}

impl<T> OutputContext<T> {
    pub fn new(strategy: Box<dyn OutputStrategy<T>>) -> Self {
        Self {
            strategy,
            process_names: None,
        }
    }

    pub fn with_process_names(mut self, process_names: Vec<(u32, String)>) -> Self {
        self.process_names = Some(process_names);
        self
    }

    pub fn display(&self, data: &[T]) -> Result<(), Box<dyn std::error::Error>> {
        self.strategy.display(data, self.process_names.as_deref())
    }
}

// 进程输出工厂函数
pub fn create_process_output(format: OutputFormat, verbose: bool) -> OutputContext<ProcessInfo> {
    use process_strategies::*;
    
    let strategy: Box<dyn OutputStrategy<ProcessInfo>> = match format {
        OutputFormat::Table => Box::new(TableStrategy { verbose }),
        OutputFormat::Json => Box::new(JsonStrategy),
        OutputFormat::Yaml => Box::new(YamlStrategy),
        OutputFormat::Csv => Box::new(CsvStrategy),
        OutputFormat::Simple => Box::new(SimpleStrategy),
        OutputFormat::Detailed => Box::new(DetailedStrategy),
    };

    OutputContext::new(strategy)
}

// 窗口输出工厂函数
pub fn create_window_output(format: OutputFormat) -> OutputContext<WindowInfo> {
    use window_strategies::*;
    
    let strategy: Box<dyn OutputStrategy<WindowInfo>> = match format {
        OutputFormat::Table => Box::new(TableStrategy),
        OutputFormat::Json => Box::new(JsonStrategy),
        OutputFormat::Yaml => Box::new(YamlStrategy),
        OutputFormat::Csv => Box::new(CsvStrategy),
        OutputFormat::Simple => Box::new(SimpleStrategy),
        OutputFormat::Detailed => Box::new(DetailedStrategy),
    };

    OutputContext::new(strategy)
}

// 保持向后兼容的接口
pub fn display_processes(
    processes: &[ProcessInfo], 
    format: OutputFormat,
    verbose: bool
) -> Result<(), Box<dyn std::error::Error>> {
    let context = create_process_output(format, verbose);
    context.display(processes)
}

pub fn display_windows(
    windows: &[WindowInfo],
    process_names: &[(u32, String)],
    format: OutputFormat,
) -> Result<(), Box<dyn std::error::Error>> {
    let context = create_window_output(format).with_process_names(process_names.to_vec());
    context.display(windows)
}

// 通用的字符串截断函数
pub fn truncate_string(s: &str, max_length: usize) -> String {
    if s.chars().count() <= max_length {
        s.to_string()
    } else {
        let mut result = String::new();
        let mut count = 0;
        
        for c in s.chars() {
            if count + c.len_utf8() <= max_length.saturating_sub(3) {
                result.push(c);
                count += 1;
            } else {
                break;
            }
        }
        
        format!("{}...", result)
    }
}
```

## 🎯 主要重构点

### 1. **策略模式设计**
```rust
trait OutputStrategy<T> {
    fn display(&self, data: &[T], process_names: Option<&[(u32, String)]>) -> Result<(), Box<dyn std::error::Error>>;
}
```
统一的策略接口，支持泛型以适应进程和窗口数据。

### 2. **分离的策略实现**
- `process_strategies` - 进程输出策略
- `window_strategies` - 窗口输出策略
- 每个格式都有专门的策略结构体

### 3. **输出上下文**
```rust
pub struct OutputContext<T> {
    strategy: Box<dyn OutputStrategy<T>>,
    process_names: Option<Vec<(u32, String)>>,
}
```
统一的输出接口，管理策略和必要的上下文数据。

### 4. **工厂函数**
```rust
pub fn create_process_output(format: OutputFormat, verbose: bool) -> OutputContext<ProcessInfo>
pub fn create_window_output(format: OutputFormat) -> OutputContext<WindowInfo>
```
简化策略对象的创建过程。

### 5. **保持向后兼容**
```rust
pub fn display_processes(...)  // 原有接口保持不变
pub fn display_windows(...)    // 原有接口保持不变
```
确保现有代码无需修改。

## 📊 重构优势

### 消除的重复：
- **格式分发逻辑** - 原来在每个display函数中重复的match语句
- **通用验证逻辑** - 窗口输出中的process_names检查
- **错误处理模式** - 统一的Result处理

### 提高的可维护性：
- **单一职责** - 每个策略只负责一种输出格式
- **开闭原则** - 易于添加新输出格式而不修改现有代码
- **更好的测试** - 可以独立测试每个策略

### 更好的扩展性：
- 添加新输出格式只需实现新的策略
- 支持新的数据类型只需实现新的策略模块
- 策略可以轻松复用和组合

这个重构显著提高了输出系统的模块化程度，使代码更易于维护、测试和扩展。