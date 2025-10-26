我来重构输出系统，使用策略模式处理不同输出格式，并提取通用的显示逻辑。以下是重构后的 `src/output.rs`：

```rust
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

// 窗口信息输出策略
struct WindowTableStrategy<'a> {
    process_names: &'a [(u32, String)],
}

impl<'a> OutputStrategy<WindowInfo> for WindowTableStrategy<'a> {
    fn display(&self, windows: &[WindowInfo]) -> Result<(), Box<dyn std::error::Error>> {
        println!("Found {} windows:", windows.len());
        println!("{:<8} {:<20} {:<30} {:<15} {:<12}", 
                 "PID", "Name", "Title", "Size", "Position");
        
        for window in windows {
            let process_name = self.get_process_name(window.pid);
            
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

impl<'a> WindowTableStrategy<'a> {
    fn get_process_name(&self, pid: u32) -> &str {
        self.process_names
            .iter()
            .find(|(process_pid, _)| *process_pid == pid)
            .map(|(_, name)| name.as_str())
            .unwrap_or("Unknown")
    }
}

struct WindowJsonStrategy<'a> {
    process_names: &'a [(u32, String)],
}

impl<'a> OutputStrategy<WindowInfo> for WindowJsonStrategy<'a> {
    fn display(&self, windows: &[WindowInfo]) -> Result<(), Box<dyn std::error::Error>> {
        let output: Vec<WindowOutput> = windows.iter()
            .map(|window| {
                let mut output = WindowOutput::from(window);
                output.name = self.get_process_name(window.pid);
                output
            })
            .collect();
        let json = serde_json::to_string_pretty(&output)?;
        println!("{}", json);
        Ok(())
    }
}

impl<'a> WindowJsonStrategy<'a> {
    fn get_process_name(&self, pid: u32) -> String {
        self.process_names
            .iter()
            .find(|(process_pid, _)| *process_pid == pid)
            .map(|(_, name)| name.clone())
            .unwrap_or_else(|| "Unknown".to_string())
    }
}

struct WindowYamlStrategy<'a> {
    process_names: &'a [(u32, String)],
}

impl<'a> OutputStrategy<WindowInfo> for WindowYamlStrategy<'a> {
    fn display(&self, windows: &[WindowInfo]) -> Result<(), Box<dyn std::error::Error>> {
        let output: Vec<WindowOutput> = windows.iter()
            .map(|window| {
                let mut output = WindowOutput::from(window);
                output.name = self.get_process_name(window.pid);
                output
            })
            .collect();
        let yaml = serde_yaml::to_string(&output)?;
        println!("{}", yaml);
        Ok(())
    }
}

impl<'a> WindowYamlStrategy<'a> {
    fn get_process_name(&self, pid: u32) -> String {
        self.process_names
            .iter()
            .find(|(process_pid, _)| *process_pid == pid)
            .map(|(_, name)| name.clone())
            .unwrap_or_else(|| "Unknown".to_string())
    }
}

struct WindowCsvStrategy<'a> {
    process_names: &'a [(u32, String)],
}

impl<'a> OutputStrategy<WindowInfo> for WindowCsvStrategy<'a> {
    fn display(&self, windows: &[WindowInfo]) -> Result<(), Box<dyn std::error::Error>> {
        let mut wtr = csv::Writer::from_writer(std::io::stdout());
        
        wtr.write_record(&["PID", "Name", "Title", "X", "Y", "Width", "Height", "Dimensions"])?;
        
        for window in windows {
            let process_name = self.get_process_name(window.pid);
            
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

impl<'a> WindowCsvStrategy<'a> {
    fn get_process_name(&self, pid: u32) -> &str {
        self.process_names
            .iter()
            .find(|(process_pid, _)| *process_pid == pid)
            .map(|(_, name)| name.as_str())
            .unwrap_or("Unknown")
    }
}

struct WindowSimpleStrategy<'a> {
    process_names: &'a [(u32, String)],
}

impl<'a> OutputStrategy<WindowInfo> for WindowSimpleStrategy<'a> {
    fn display(&self, windows: &[WindowInfo]) -> Result<(), Box<dyn std::error::Error>> {
        for window in windows {
            let process_name = self.get_process_name(window.pid);
            
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

impl<'a> WindowSimpleStrategy<'a> {
    fn get_process_name(&self, pid: u32) -> &str {
        self.process_names
            .iter()
            .find(|(process_pid, _)| *process_pid == pid)
            .map(|(_, name)| name.as_str())
            .unwrap_or("Unknown")
    }
}

struct WindowDetailedStrategy<'a> {
    process_names: &'a [(u32, String)],
}

impl<'a> OutputStrategy<WindowInfo> for WindowDetailedStrategy<'a> {
    fn display(&self, windows: &[WindowInfo]) -> Result<(), Box<dyn std::error::Error>> {
        for (i, window) in windows.iter().enumerate() {
            let process_name = self.get_process_name(window.pid);
            
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

impl<'a> WindowDetailedStrategy<'a> {
    fn get_process_name(&self, pid: u32) -> &str {
        self.process_names
            .iter()
            .find(|(process_pid, _)| *process_pid == pid)
            .map(|(_, name)| name.as_str())
            .unwrap_or("Unknown")
    }
}

// 公共接口函数
pub fn display_processes(
    processes: &[&ProcessInfo], 
    format: OutputFormat,
    verbose: bool
) -> Result<(), Box<dyn std::error::Error>> {
    match format {
        OutputFormat::Table => ProcessTableStrategy { verbose }.display(processes),
        OutputFormat::Json => ProcessJsonStrategy.display(processes),
        OutputFormat::Yaml => ProcessYamlStrategy.display(processes),
        OutputFormat::Csv => ProcessCsvStrategy.display(processes),
        OutputFormat::Simple => ProcessSimpleStrategy.display(processes),
        OutputFormat::Detailed => ProcessDetailedStrategy.display(processes),
    }
}

pub fn display_windows(
    windows: &[WindowInfo],
    process_names: &[(u32, String)],
    format: OutputFormat,
) -> Result<(), Box<dyn std::error::Error>> {
    match format {
        OutputFormat::Table => WindowTableStrategy { process_names }.display(windows),
        OutputFormat::Json => WindowJsonStrategy { process_names }.display(windows),
        OutputFormat::Yaml => WindowYamlStrategy { process_names }.display(windows),
        OutputFormat::Csv => WindowCsvStrategy { process_names }.display(windows),
        OutputFormat::Simple => WindowSimpleStrategy { process_names }.display(windows),
        OutputFormat::Detailed => WindowDetailedStrategy { process_names }.display(windows),
    }
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

**主要重构改进：**

1. **策略模式实现**：
   - 定义了 `OutputStrategy<T>` trait，统一输出接口
   - 为每种输出格式创建了独立的策略结构体

2. **进程输出策略**：
   - `ProcessTableStrategy` - 表格输出（支持verbose模式）
   - `ProcessJsonStrategy` - JSON输出
   - `ProcessYamlStrategy` - YAML输出
   - `ProcessCsvStrategy` - CSV输出
   - `ProcessSimpleStrategy` - 简单输出
   - `ProcessDetailedStrategy` - 详细输出

3. **窗口输出策略**：
   - 所有窗口策略都包含进程名称查找功能
   - 提取了通用的 `get_process_name` 方法到各个策略中

4. **代码组织改进**：
   - 每种输出格式的逻辑被封装在独立的结构体中
   - 公共接口函数 `display_processes` 和 `display_windows` 保持不变，保持向后兼容
   - 减少了函数间的耦合

5. **可维护性提升**：
   - 添加新的输出格式只需实现 `OutputStrategy` trait
   - 修改特定输出格式不会影响其他格式
   - 代码结构更加清晰，职责分离

这种重构使得输出系统更加模块化，易于扩展和维护，同时保持了原有的功能完整性。