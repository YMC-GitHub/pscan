mod types;
mod cli;
mod process;
mod window;
mod output;

use std::process::exit;
use output::{OutputFormat, display_processes, display_windows};
use cli::{parse_args, SubCommand};
use process::{get_processes, filter_processes};
use window::get_all_windows_with_size;
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
        None => {
            // Handle normal process listing
            handle_process_command(config);
        }
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