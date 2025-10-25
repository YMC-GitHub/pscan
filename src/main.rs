mod output;

use clap::{Arg, Command};
use sysinfo::System;
use std::process;
use output::{OutputFormat, display_processes};

#[derive(Debug)]
struct ProcessInfo {
    pid: String,
    name: String,
    title: String,
    memory_usage: u64,
    has_window: bool,
}

fn main() {
    let matches = Command::new("Process Filter")
        .version(env!("CARGO_PKG_VERSION"))
        .author(env!("CARGO_PKG_AUTHORS"))
        .about(env!("CARGO_PKG_DESCRIPTION"))
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
            Arg::new("has_window")
                .long("has-window")
                .action(clap::ArgAction::SetTrue)
                .help("Show only processes with windows")
        )
        .arg(
            Arg::new("no_window")
                .long("no-window")
                .action(clap::ArgAction::SetTrue)
                .help("Show only processes without windows")
                .conflicts_with("has_window")
        )
        .arg(
            Arg::new("format")
                .short('f')
                .long("format")
                .value_name("FORMAT")
                .value_parser(clap::value_parser!(OutputFormat))
                .default_value("table")
                .help("Output format")
        )
        .arg(
            Arg::new("verbose")
                .short('v')
                .long("verbose")
                .action(clap::ArgAction::SetTrue)
                .help("Show detailed information")
        )
        .get_matches();

    // Get filter criteria
    let pid_filter = matches.get_one::<String>("pid").map(|s| s.as_str());
    let name_filter = matches.get_one::<String>("name").map(|s| s.as_str());
    let title_filter = matches.get_one::<String>("title").map(|s| s.as_str());
    let has_window_filter = matches.get_flag("has_window");
    let no_window_filter = matches.get_flag("no_window");
    let format = matches.get_one::<OutputFormat>("format").unwrap();
    let verbose = matches.get_flag("verbose");

    // Get process list
    let processes = get_processes();

    // Filter processes
    let filtered_processes: Vec<&ProcessInfo> = processes
        .iter()
        .filter(|p| {
            // PID filter
            if let Some(pid) = pid_filter {
                if p.pid != pid {
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
        .collect();

    // Display results
    if filtered_processes.is_empty() {
        eprintln!("No matching processes found");
        process::exit(1);
    }

    if let Err(e) = display_processes(&filtered_processes, format.clone(), verbose) {
        eprintln!("Output error: {}", e);
        process::exit(1);
    }
}

fn get_processes() -> Vec<ProcessInfo> {
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

fn get_process_title_fallback(process: &sysinfo::Process) -> String {
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

// Get all visible windows with their PIDs and titles
fn get_all_windows() -> Vec<(u32, String)> {
    let mut windows = Vec::new();
    
    #[cfg(windows)]
    {
        use std::ffi::OsString;
        use std::os::windows::ffi::OsStringExt;
        use windows::Win32::Foundation::{HWND, BOOL, TRUE, LPARAM};
        use windows::Win32::UI::WindowsAndMessaging::{
            EnumWindows, GetWindowThreadProcessId, IsWindowVisible, GetWindowTextW
        };
        
        unsafe extern "system" fn enum_windows_proc(hwnd: HWND, lparam: LPARAM) -> BOOL {
            let windows_ptr = lparam.0 as *mut Vec<(u32, String)>;
            
            if IsWindowVisible(hwnd).as_bool() {
                // Get window title
                let mut title_vec = vec![0u16; 512];
                let title_len = GetWindowTextW(hwnd, &mut title_vec);
                
                if title_len > 0 {
                    title_vec.truncate(title_len as usize);
                    let title_os = OsString::from_wide(&title_vec);
                    let title = title_os.to_string_lossy().to_string();
                    
                    // Only include non-empty titles
                    if !title.trim().is_empty() {
                        let mut pid: u32 = 0;
                        GetWindowThreadProcessId(hwnd, Some(&mut pid));
                        
                        if pid != 0 {
                            unsafe {
                                (*windows_ptr).push((pid, title));
                            }
                        }
                    }
                }
            }
            
            TRUE // Continue enumeration
        }
        
        unsafe {
            EnumWindows(
                Some(enum_windows_proc),
                LPARAM(&mut windows as *mut _ as isize),
            ).ok(); // Ignore errors for now
        }
    }
    
    #[cfg(not(windows))]
    {
        // On non-Windows systems, we'll use a simpler approach
        // This is a placeholder - you might want to implement X11 or other window system support
        println!("Warning: Window detection is limited on non-Windows systems");
    }
    
    windows
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_truncate_string() {
        assert_eq!(output::truncate_string("hello", 5), "hello");
        assert_eq!(output::truncate_string("hello world", 8), "hello...");
        assert_eq!(output::truncate_string("hi", 10), "hi");
    }
}