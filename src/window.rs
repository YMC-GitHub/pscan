// Get all visible windows with their PIDs and titles
pub fn get_all_windows() -> Vec<(u32, String)> {
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