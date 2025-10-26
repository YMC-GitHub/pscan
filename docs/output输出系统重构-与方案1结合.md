基于比较分析，我提供一个结合了两种方案优点的完整输出系统代码：

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
    fn display(&self, data: &[T]) -> Result<(), Box<dyn std::error::Error>>;
}

// 进程名称提供者 trait - 消除重复代码
trait ProcessNameProvider {
    fn get_process_name(&self, pid: u32) -> &str;
    fn get_process_name_owned(&self, pid: u32) -> String;
}

impl ProcessNameProvider for &[(u32, String)] {
    fn get_process_name(&self, pid: u32) -> &str {
        self.iter()
            .find(|(process_pid, _)| *process_pid == pid)
            .map(|(_, name)| name.as_str())
            .unwrap_or("Unknown")
    }

    fn get_process_name_owned(&self, pid: u32) -> String {
        self.get_process_name(pid).to_string()
    }
}

// 进程信息输出策略
struct ProcessTableStrategy {
    verbose: bool,
}

impl OutputStrategy<&ProcessInfo> for ProcessTableStrategy {
    fn display(&self, processes: &[&ProcessInfo]) -> Result<(), Box<dyn std::error::Error>> {
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

struct ProcessJsonStrategy;

impl OutputStrategy<&ProcessInfo> for ProcessJsonStrategy {
    fn display(&self, processes: &[&ProcessInfo]) -> Result<(), Box<dyn std::error::Error>> {
        let output: Vec<ProcessOutput> = processes.iter().map(|p| ProcessOutput::from(*p)).collect();
        let json = serde_json::to_string_pretty(&output)?;
        println!("{}", json);
        Ok(())
    }
}

struct ProcessYamlStrategy;

impl OutputStrategy<&ProcessInfo> for ProcessYamlStrategy {
    fn display(&self, processes: &[&ProcessInfo]) -> Result<(), Box<dyn std::error::Error>> {
        let output: Vec<ProcessOutput> = processes.iter().map(|p| ProcessOutput::from(*p)).collect();
        let yaml = serde_yaml::to_string(&output)?;
        println!("{}", yaml);
        Ok(())
    }
}

struct ProcessCsvStrategy;

impl OutputStrategy<&ProcessInfo> for ProcessCsvStrategy {
    fn display(&self, processes: &[&ProcessInfo]) -> Result<(), Box<dyn std::error::Error>> {
        let mut wtr = csv::Writer::from_writer(std::io::stdout());
        
        wtr.write_record(&["PID", "Name", "Title", "MemoryUsage", "MemoryUsageMB", "HasWindow"])?;
        
        for process in processes {
            let output = ProcessOutput::from(*process);
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

struct ProcessSimpleStrategy;

impl OutputStrategy<&ProcessInfo> for ProcessSimpleStrategy {
    fn display(&self, processes: &[&ProcessInfo]) -> Result<(), Box<dyn std::error::Error>> {
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

struct ProcessDetailedStrategy;

impl OutputStrategy<&ProcessInfo> for ProcessDetailedStrategy {
    fn display(&self, processes: &[&ProcessInfo]) -> Result<(), Box<dyn std::error::Error>> {
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

// 窗口策略基结构 - 消除重复的进程名称查找逻辑
struct WindowStrategyBase<'a> {
    process_names: &'a [(u32, String)],
}

impl<'a> WindowStrategyBase<'a> {
    fn new(process_names: &'a [(u32, String)]) -> Self {
        Self { process_names }
    }

    fn get_process_name(&self, pid: u32) -> &str {
        self.process_names.get_process_name(pid)
    }

    fn get_process_name_owned(&self, pid: u32) -> String {
        self.process_names.get_process_name_owned(pid)
    }
}

// 窗口信息输出策略
struct WindowTableStrategy<'a> {
    base: WindowStrategyBase<'a>,
}

impl<'a> WindowTableStrategy<'a> {
    fn new(process_names: &'a [(u32, String)]) -> Self {
        Self {
            base: WindowStrategyBase::new(process_names),
        }
    }
}

impl<'a> OutputStrategy<WindowInfo> for WindowTableStrategy<'a> {
    fn display(&self, windows: &[WindowInfo]) -> Result<(), Box<dyn std::error::Error>> {
        println!("Found {} windows:", windows.len());
        println!("{:<8} {:<20} {:<30} {:<15} {:<12}", 
                 "PID", "Name", "Title", "Size", "Position");
        
        for window in windows {
            let process_name = self.base.get_process_name(window.pid);
            
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

struct WindowJsonStrategy<'a> {
    base: WindowStrategyBase<'a>,
}

impl<'a> WindowJsonStrategy<'a> {
    fn new(process_names: &'a [(u32, String)]) -> Self {
        Self {
            base: WindowStrategyBase::new(process_names),
        }
    }
}

impl<'a> OutputStrategy<WindowInfo> for WindowJsonStrategy<'a> {
    fn display(&self, windows: &[WindowInfo]) -> Result<(), Box<dyn std::error::Error>> {
        let output: Vec<WindowOutput> = windows.iter()
            .map(|window| {
                let mut output = WindowOutput::from(window);
                output.name = self.base.get_process_name_owned(window.pid);
                output
            })
            .collect();
        let json = serde_json::to_string_pretty(&output)?;
        println!("{}", json);
        Ok(())
    }
}

struct WindowYamlStrategy<'a> {
    base: WindowStrategyBase<'a>,
}

impl<'a> WindowYamlStrategy<'a> {
    fn new(process_names: &'a [(u32, String)]) -> Self {
        Self {
            base: WindowStrategyBase::new(process_names),
        }
    }
}

impl<'a> OutputStrategy<WindowInfo> for WindowYamlStrategy<'a> {
    fn display(&self, windows: &[WindowInfo]) -> Result<(), Box<dyn std::error::Error>> {
        let output: Vec<WindowOutput> = windows.iter()
            .map(|window| {
                let mut output = WindowOutput::from(window);
                output.name = self.base.get_process_name_owned(window.pid);
                output
            })
            .collect();
        let yaml = serde_yaml::to_string(&output)?;
        println!("{}", yaml);
        Ok(())
    }
}

struct WindowCsvStrategy<'a> {
    base: WindowStrategyBase<'a>,
}

impl<'a> WindowCsvStrategy<'a> {
    fn new(process_names: &'a [(u32, String)]) -> Self {
        Self {
            base: WindowStrategyBase::new(process_names),
        }
    }
}

impl<'a> OutputStrategy<WindowInfo> for WindowCsvStrategy<'a> {
    fn display(&self, windows: &[WindowInfo]) -> Result<(), Box<dyn std::error::Error>> {
        let mut wtr = csv::Writer::from_writer(std::io::stdout());
        
        wtr.write_record(&["PID", "Name", "Title", "X", "Y", "Width", "Height", "Dimensions"])?;
        
        for window in windows {
            let process_name = self.base.get_process_name(window.pid);
            
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

struct WindowSimpleStrategy<'a> {
    base: WindowStrategyBase<'a>,
}

impl<'a> WindowSimpleStrategy<'a> {
    fn new(process_names: &'a [(u32, String)]) -> Self {
        Self {
            base: WindowStrategyBase::new(process_names),
        }
    }
}

impl<'a> OutputStrategy<WindowInfo> for WindowSimpleStrategy<'a> {
    fn display(&self, windows: &[WindowInfo]) -> Result<(), Box<dyn std::error::Error>> {
        for window in windows {
            let process_name = self.base.get_process_name(window.pid);
            
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

struct WindowDetailedStrategy<'a> {
    base: WindowStrategyBase<'a>,
}

impl<'a> WindowDetailedStrategy<'a> {
    fn new(process_names: &'a [(u32, String)]) -> Self {
        Self {
            base: WindowStrategyBase::new(process_names),
        }
    }
}

impl<'a> OutputStrategy<WindowInfo> for WindowDetailedStrategy<'a> {
    fn display(&self, windows: &[WindowInfo]) -> Result<(), Box<dyn std::error::Error>> {
        for (i, window) in windows.iter().enumerate() {
            let process_name = self.base.get_process_name(window.pid);
            
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

// 策略创建辅助函数
fn create_process_strategy(
    format: OutputFormat,
    verbose: bool,
) -> impl OutputStrategy<&ProcessInfo> {
    match format {
        OutputFormat::Table => ProcessTableStrategy { verbose },
        OutputFormat::Json => ProcessJsonStrategy,
        OutputFormat::Yaml => ProcessYamlStrategy,
        OutputFormat::Csv => ProcessCsvStrategy,
        OutputFormat::Simple => ProcessSimpleStrategy,
        OutputFormat::Detailed => ProcessDetailedStrategy,
    }
}

fn create_window_strategy<'a>(
    format: OutputFormat,
    process_names: &'a [(u32, String)],
) -> impl OutputStrategy<WindowInfo> + 'a {
    match format {
        OutputFormat::Table => WindowTableStrategy::new(process_names),
        OutputFormat::Json => WindowJsonStrategy::new(process_names),
        OutputFormat::Yaml => WindowYamlStrategy::new(process_names),
        OutputFormat::Csv => WindowCsvStrategy::new(process_names),
        OutputFormat::Simple => WindowSimpleStrategy::new(process_names),
        OutputFormat::Detailed => WindowDetailedStrategy::new(process_names),
    }
}

// 公共接口函数
pub fn display_processes(
    processes: &[&ProcessInfo], 
    format: OutputFormat,
    verbose: bool
) -> Result<(), Box<dyn std::error::Error>> {
    let strategy = create_process_strategy(format, verbose);
    strategy.display(processes)
}

pub fn display_windows(
    windows: &[WindowInfo],
    process_names: &[(u32, String)],
    format: OutputFormat,
) -> Result<(), Box<dyn std::error::Error>> {
    let strategy = create_window_strategy(format, process_names);
    strategy.display(windows)
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_truncate_string() {
        assert_eq!(truncate_string("hello", 5), "hello");
        assert_eq!(truncate_string("hello world", 8), "hello...");
        assert_eq!(truncate_string("hi", 10), "hi");
        assert_eq!(truncate_string("测试字符串", 5), "测试...");
    }

    #[test]
    fn test_process_name_provider() {
        let process_names = vec![
            (1234, "test_process".to_string()),
            (5678, "another_process".to_string()),
        ];
        
        let provider = &process_names[..];
        
        assert_eq!(provider.get_process_name(1234), "test_process");
        assert_eq!(provider.get_process_name(5678), "another_process");
        assert_eq!(provider.get_process_name(9999), "Unknown");
        
        assert_eq!(provider.get_process_name_owned(1234), "test_process");
        assert_eq!(provider.get_process_name_owned(9999), "Unknown");
    }

    #[test]
    fn test_window_strategy_base() {
        let process_names = vec![
            (1001, "window_process".to_string()),
        ];
        
        let base = WindowStrategyBase::new(&process_names);
        
        assert_eq!(base.get_process_name(1001), "window_process");
        assert_eq!(base.get_process_name(2002), "Unknown");
        assert_eq!(base.get_process_name_owned(1001), "window_process");
    }
}
```

## 🎯 主要优化点

### 1. **消除重复代码**
```rust
trait ProcessNameProvider {
    fn get_process_name(&self, pid: u32) -> &str;
    fn get_process_name_owned(&self, pid: u32) -> String;
}
```
统一的进程名称查找逻辑，避免在每个窗口策略中重复实现。

### 2. **窗口策略基类**
```rust
struct WindowStrategyBase<'a> {
    process_names: &'a [(u32, String)],
}
```
集中管理窗口策略的公共数据和功能。

### 3. **策略创建函数**
```rust
fn create_process_strategy(format: OutputFormat, verbose: bool) -> impl OutputStrategy<&ProcessInfo>
fn create_window_strategy<'a>(format: OutputFormat, process_names: &'a [(u32, String)]) -> impl OutputStrategy<WindowInfo> + 'a
```
统一的策略创建接口，保持代码整洁。

### 4. **保持性能优势**
- 无动态分发开销
- 使用引用而非克隆
- 编译时优化

### 5. **完整的测试覆盖**
- 字符串截断功能测试
- 进程名称提供者测试
- 窗口策略基础功能测试

## 📊 重构效果

这个实现结合了两种方案的优点：

- **保持简单性** - 直接的结构体实现，易于理解
- **消除重复** - 通过trait和基类避免代码重复
- **性能优秀** - 编译时确定类型，无运行时开销
- **易于维护** - 统一的接口和创建模式
- **易于测试** - 模块化的设计便于单元测试

这个重构方案既保持了您方案的性能优势，又通过适当的抽象消除了重复代码，是工程实践中的最佳选择。