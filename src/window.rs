use crate::types::{WindowInfo, WindowRect};

// Get all visible windows with their PIDs, titles, and dimensions
pub fn get_all_windows_with_size() -> Vec<WindowInfo> {
    let mut windows = Vec::new();
    
    #[cfg(windows)]
    {
        use std::ffi::OsString;
        use std::os::windows::ffi::OsStringExt;
        use windows::Win32::Foundation::{HWND, BOOL, LPARAM, RECT};
        use windows::Win32::UI::WindowsAndMessaging::{
            EnumWindows, GetWindowThreadProcessId, IsWindowVisible, GetWindowTextW, GetWindowRect
        };
        
        unsafe extern "system" fn enum_windows_proc(hwnd: HWND, lparam: LPARAM) -> BOOL {
            let windows_ptr = lparam.0 as *mut Vec<WindowInfo>;
            
            if IsWindowVisible(hwnd).into() {
                // Get window title
                let mut title_vec = vec![0u16; 512];
                let title_len = GetWindowTextW(hwnd, &mut title_vec);
                
                if title_len > 0 {
                    title_vec.truncate(title_len as usize);
                    let title_os = OsString::from_wide(&title_vec);
                    let title = title_os.to_string_lossy().to_string();
                    
                    // Only include non-empty titles
                    if !title.trim().is_empty() {
                        // Get window rectangle
                        let mut rect = RECT::default();
                        if GetWindowRect(hwnd, &mut rect).is_ok() {
                            let window_rect = WindowRect::new(
                                rect.left,
                                rect.top,
                                rect.right - rect.left,
                                rect.bottom - rect.top
                            );
                            
                            let mut pid: u32 = 0;
                            GetWindowThreadProcessId(hwnd, Some(&mut pid));
                            
                            if pid != 0 {
                                unsafe {
                                    (*windows_ptr).push(WindowInfo {
                                        pid,
                                        title,
                                        rect: window_rect,
                                    });
                                }
                            }
                        }
                    }
                }
            }
            
            true.into() // Continue enumeration
        }
        
        unsafe {
            let _ = EnumWindows(
                Some(enum_windows_proc),
                LPARAM(&mut windows as *mut _ as isize),
            );
        }
    }
    
    #[cfg(not(windows))]
    {
        // On non-Windows systems, we'll use a simpler approach
        println!("Warning: Window size detection is limited on non-Windows systems");
    }
    
    windows
}

// Keep the original function for basic window detection
pub fn get_all_windows() -> Vec<(u32, String)> {
    get_all_windows_with_size()
        .into_iter()
        .map(|window| (window.pid, window.title))
        .collect()
}

// Window manipulation functions
#[cfg(windows)]
pub mod manipulation {
    use windows::Win32::Foundation::{HWND, BOOL, LPARAM};
    use windows::Win32::UI::WindowsAndMessaging::{
        ShowWindow, SW_MINIMIZE, SW_MAXIMIZE, SW_RESTORE, 
        EnumWindows, GetWindowThreadProcessId, IsWindowVisible, GetWindowTextW
    };
    use std::ffi::OsString;
    use std::os::windows::ffi::OsStringExt;

    pub struct WindowHandle {
        pub hwnd: HWND,
        pub pid: u32,
        pub title: String,
    }

    impl WindowHandle {
        pub fn minimize(&self) -> Result<(), String> {
            unsafe {
                ShowWindow(self.hwnd, SW_MINIMIZE);
            }
            Ok(())
        }

        pub fn maximize(&self) -> Result<(), String> {
            unsafe {
                ShowWindow(self.hwnd, SW_MAXIMIZE);
            }
            Ok(())
        }

        pub fn restore(&self) -> Result<(), String> {
            unsafe {
                ShowWindow(self.hwnd, SW_RESTORE);
            }
            Ok(())
        }
    }

    pub fn find_windows(
        pid_filter: &Option<String>,
        name_filter: &Option<String>,
        title_filter: &Option<String>,
        process_names: &[(u32, String)],
    ) -> Vec<WindowHandle> {
        let mut windows = Vec::new();
        
        unsafe extern "system" fn enum_windows_proc(hwnd: HWND, lparam: LPARAM) -> BOOL {
            let windows_ptr = lparam.0 as *mut Vec<WindowHandle>;
            
            if IsWindowVisible(hwnd).into() {
                // Get window title
                let mut title_vec = vec![0u16; 512];
                let title_len = GetWindowTextW(hwnd, &mut title_vec);
                
                if title_len > 0 {
                    title_vec.truncate(title_len as usize);
                    let title_os = OsString::from_wide(&title_vec);
                    let title = title_os.to_string_lossy().to_string();
                    
                    if !title.trim().is_empty() {
                        let mut pid: u32 = 0;
                        GetWindowThreadProcessId(hwnd, Some(&mut pid));
                        
                        if pid != 0 {
                            unsafe {
                                (*windows_ptr).push(WindowHandle {
                                    hwnd,
                                    pid,
                                    title,
                                });
                            }
                        }
                    }
                }
            }
            
            true.into()
        }
        
        unsafe {
            let _ = EnumWindows(
                Some(enum_windows_proc),
                LPARAM(&mut windows as *mut _ as isize),
            );
        }

        // Apply filters - explicitly specify the type for the closure parameter
        windows.into_iter()
            .filter(|window: &WindowHandle| {
                // PID filter
                if let Some(pid) = pid_filter {
                    if window.pid.to_string() != *pid {
                        return false;
                    }
                }

                // Name filter
                if let Some(name) = name_filter {
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
                if let Some(title) = title_filter {
                    if !window.title.to_lowercase().contains(&title.to_lowercase()) {
                        return false;
                    }
                }

                true
            })
            .collect()
    }

    pub fn minimize_windows(
        pid_filter: &Option<String>,
        name_filter: &Option<String>,
        title_filter: &Option<String>,
        process_names: &[(u32, String)],
        all: bool,
    ) -> Result<usize, String> {
        let windows = find_windows(pid_filter, name_filter, title_filter, process_names);
        
        if windows.is_empty() {
            return Err("No matching windows found".to_string());
        }

        if !all && windows.len() > 1 {
            return Err(format!("Multiple windows found ({}). Use --all to minimize all matching windows", windows.len()));
        }

        let mut count = 0;
        for window in windows {
            if let Err(e) = window.minimize() {
                eprintln!("Failed to minimize window {} (PID: {}): {}", window.title, window.pid, e);
            } else {
                println!("Minimized: {} (PID: {})", window.title, window.pid);
                count += 1;
            }
        }

        Ok(count)
    }

    pub fn maximize_windows(
        pid_filter: &Option<String>,
        name_filter: &Option<String>,
        title_filter: &Option<String>,
        process_names: &[(u32, String)],
        all: bool,
    ) -> Result<usize, String> {
        let windows = find_windows(pid_filter, name_filter, title_filter, process_names);
        
        if windows.is_empty() {
            return Err("No matching windows found".to_string());
        }

        if !all && windows.len() > 1 {
            return Err(format!("Multiple windows found ({}). Use --all to maximize all matching windows", windows.len()));
        }

        let mut count = 0;
        for window in windows {
            if let Err(e) = window.maximize() {
                eprintln!("Failed to maximize window {} (PID: {}): {}", window.title, window.pid, e);
            } else {
                println!("Maximized: {} (PID: {})", window.title, window.pid);
                count += 1;
            }
        }

        Ok(count)
    }

    pub fn restore_windows(
        pid_filter: &Option<String>,
        name_filter: &Option<String>,
        title_filter: &Option<String>,
        process_names: &[(u32, String)],
        all: bool,
    ) -> Result<usize, String> {
        let windows = find_windows(pid_filter, name_filter, title_filter, process_names);
        
        if windows.is_empty() {
            return Err("No matching windows found".to_string());
        }

        if !all && windows.len() > 1 {
            return Err(format!("Multiple windows found ({}). Use --all to restore all matching windows", windows.len()));
        }

        let mut count = 0;
        for window in windows {
            if let Err(e) = window.restore() {
                eprintln!("Failed to restore window {} (PID: {}): {}", window.title, window.pid, e);
            } else {
                println!("Restored: {} (PID: {})", window.title, window.pid);
                count += 1;
            }
        }

        Ok(count)
    }
}

#[cfg(not(windows))]
pub mod manipulation {
    use super::*;

    pub fn minimize_windows(
        _pid_filter: &Option<String>,
        _name_filter: &Option<String>,
        _title_filter: &Option<String>,
        _process_names: &[(u32, String)],
        _all: bool,
    ) -> Result<usize, String> {
        Err("Window manipulation is only supported on Windows".to_string())
    }

    pub fn maximize_windows(
        _pid_filter: &Option<String>,
        _name_filter: &Option<String>,
        _title_filter: &Option<String>,
        _process_names: &[(u32, String)],
        _all: bool,
    ) -> Result<usize, String> {
        Err("Window manipulation is only supported on Windows".to_string())
    }

    pub fn restore_windows(
        _pid_filter: &Option<String>,
        _name_filter: &Option<String>,
        _title_filter: &Option<String>,
        _process_names: &[(u32, String)],
        _all: bool,
    ) -> Result<usize, String> {
        Err("Window manipulation is only supported on Windows".to_string())
    }
}