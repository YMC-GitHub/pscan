use sysinfo::{System, Process};
use crate::types::ProcessInfo;
use crate::window::get_all_windows;

pub fn get_processes() -> Vec<ProcessInfo> {
    let mut system = System::new_all();
    
    // Refresh process information
    system.refresh_all();
    
    // First get all window information
    let window_info = get_all_windows();
    
    let mut processes = Vec::new();

    for (pid, process) in system.processes() {
        let pid_str = pid.to_string();
        let pid_u32 = pid.as_u32();
        
        // Check if this process has windows and get the title
        let (has_window, title) = if let Some((_window_pid, window_title)) = window_info.iter()
            .find(|(wp, _)| *wp == pid_u32) {
            (true, window_title.clone())
        } else {
            (false, get_process_title_fallback(process))
        };
        
        let process_info = ProcessInfo {
            pid: pid_str,
            name: process.name().to_string(),
            title,
            memory_usage: process.memory(),
            has_window,
        };
        
        processes.push(process_info);
    }

    processes
}

fn get_process_title_fallback(process: &Process) -> String {
    // Use command line arguments as fallback title
    let cmd = process.cmd();
    if !cmd.is_empty() {
        return cmd.join(" ");
    }
    
    // If no command line arguments, use executable path
    if let Some(exe) = process.exe().and_then(|p| p.to_str()) {
        return exe.to_string();
    }
    
    "No Title".to_string()
}

pub fn filter_processes<'a>(
    processes: &'a [ProcessInfo],
    pid_filter: &Option<String>,
    name_filter: &Option<String>,
    title_filter: &Option<String>,
    has_window_filter: bool,
    no_window_filter: bool,
) -> Vec<&'a ProcessInfo> {
    processes
        .iter()
        .filter(|p| {
            // PID filter
            if let Some(pid) = pid_filter {
                if p.pid != *pid {
                    return false;
                }
            }

            // Name filter
            if let Some(name) = name_filter {
                if !p.name.to_lowercase().contains(&name.to_lowercase()) {
                    return false;
                }
            }

            // Title filter
            if let Some(title) = title_filter {
                if !p.title.to_lowercase().contains(&title.to_lowercase()) {
                    return false;
                }
            }

            // Window presence filter
            if has_window_filter && !p.has_window {
                return false;
            }

            if no_window_filter && p.has_window {
                return false;
            }

            true
        })
        .collect()
}