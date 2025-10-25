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

pub fn display_processes(
    processes: &[&ProcessInfo], 
    format: OutputFormat,
    verbose: bool
) -> Result<(), Box<dyn std::error::Error>> {
    match format {
        OutputFormat::Table => display_table(processes, verbose),
        OutputFormat::Json => display_json(processes),
        OutputFormat::Yaml => display_yaml(processes),
        OutputFormat::Csv => display_csv(processes),
        OutputFormat::Simple => display_simple(processes),
        OutputFormat::Detailed => display_detailed(processes),
    }
}

fn display_table(processes: &[&ProcessInfo], verbose: bool) -> Result<(), Box<dyn std::error::Error>> {
    println!("Found {} matching processes:", processes.len());
    
    if verbose {
        println!("{:<8} {:<20} {:<30} {:<12} {}", 
                 "PID", "Name", "Title", "Memory", "Window");
    } else {
        println!("{:<8} {:<20} {:<30} {}", 
                 "PID", "Name", "Title", "Memory");
    }

    for process in processes {
        let memory_mb = process.memory_usage as f64 / 1024.0 / 1024.0;
        
        if verbose {
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

        if verbose {
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

fn display_json(processes: &[&ProcessInfo]) -> Result<(), Box<dyn std::error::Error>> {
    let output: Vec<ProcessOutput> = processes.iter().map(|p| ProcessOutput::from(*p)).collect();
    let json = serde_json::to_string_pretty(&output)?;
    println!("{}", json);
    Ok(())
}

fn display_yaml(processes: &[&ProcessInfo]) -> Result<(), Box<dyn std::error::Error>> {
    let output: Vec<ProcessOutput> = processes.iter().map(|p| ProcessOutput::from(*p)).collect();
    let yaml = serde_yaml::to_string(&output)?;
    println!("{}", yaml);
    Ok(())
}

fn display_csv(processes: &[&ProcessInfo]) -> Result<(), Box<dyn std::error::Error>> {
    let mut wtr = csv::Writer::from_writer(std::io::stdout());
    
    // Write header
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

fn display_simple(processes: &[&ProcessInfo]) -> Result<(), Box<dyn std::error::Error>> {
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

fn display_detailed(processes: &[&ProcessInfo]) -> Result<(), Box<dyn std::error::Error>> {
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

// Window output functions
pub fn display_windows(
    windows: &[WindowInfo],
    process_names: &[(u32, String)],
    format: OutputFormat,
) -> Result<(), Box<dyn std::error::Error>> {
    match format {
        OutputFormat::Table => display_window_table(windows, process_names),
        OutputFormat::Json => display_window_json(windows, process_names),
        OutputFormat::Yaml => display_window_yaml(windows, process_names),
        OutputFormat::Csv => display_window_csv(windows, process_names),
        OutputFormat::Simple => display_window_simple(windows, process_names),
        OutputFormat::Detailed => display_window_detailed(windows, process_names),
    }
}

fn display_window_table(windows: &[WindowInfo], process_names: &[(u32, String)]) -> Result<(), Box<dyn std::error::Error>> {
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

fn display_window_json(windows: &[WindowInfo], process_names: &[(u32, String)]) -> Result<(), Box<dyn std::error::Error>> {
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

fn display_window_yaml(windows: &[WindowInfo], process_names: &[(u32, String)]) -> Result<(), Box<dyn std::error::Error>> {
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

fn display_window_csv(windows: &[WindowInfo], process_names: &[(u32, String)]) -> Result<(), Box<dyn std::error::Error>> {
    let mut wtr = csv::Writer::from_writer(std::io::stdout());
    
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

fn display_window_simple(windows: &[WindowInfo], process_names: &[(u32, String)]) -> Result<(), Box<dyn std::error::Error>> {
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

fn display_window_detailed(windows: &[WindowInfo], process_names: &[(u32, String)]) -> Result<(), Box<dyn std::error::Error>> {
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

pub fn truncate_string(s: &str, max_length: usize) -> String {
    if s.chars().count() <= max_length {
        s.to_string()
    } else {
        // 使用字符迭代器来安全地截断字符串
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