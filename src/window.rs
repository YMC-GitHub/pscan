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