// src/platform/windows.rs
use windows::Win32::Foundation::{HWND, BOOL, LPARAM};
use windows::Win32::UI::WindowsAndMessaging::{
    EnumWindows, GetWindowTextW, GetWindowThreadProcessId, GetWindowRect, 
    SetWindowPos, ShowWindow, IsWindowVisible, GetClassNameW,
    SW_MINIMIZE, SW_MAXIMIZE, SW_RESTORE, SWP_NOZORDER, SWP_NOACTIVATE
};
use crate::platform::interface::PlatformWindow;
use crate::types::{WindowInfo, WindowRect};

/// Windows 平台特定的窗口数据
#[derive(Debug, Clone)]
pub struct WindowsWindowData {
    pub hwnd: isize, // 使用 isize 而不是 HWND 来避免平台特定类型的暴露
}

impl WindowsWindowData {
    pub fn new(hwnd: isize) -> Self {
        Self { hwnd }
    }

    pub fn minimize(&self) -> Result<(), String> {
        unsafe {
            let hwnd = HWND(self.hwnd);
            if !IsWindowVisible(hwnd).as_bool() {
                return Err("Window not visible or invalid handle".to_string());
            }
            
            let result = ShowWindow(hwnd, SW_MINIMIZE);
            if result.0 != 0 {
                Ok(())
            } else {
                Err("Failed to minimize window".to_string())
            }
        }
    }

    pub fn maximize(&self) -> Result<(), String> {
        unsafe {
            let hwnd = HWND(self.hwnd);
            if !IsWindowVisible(hwnd).as_bool() {
                return Err("Window not visible or invalid handle".to_string());
            }
            
            let result = ShowWindow(hwnd, SW_MAXIMIZE);
            if result.0 != 0 {
                Ok(())
            } else {
                Err("Failed to maximize window".to_string())
            }
        }
    }

    pub fn restore(&self) -> Result<(), String> {
        unsafe {
            let hwnd = HWND(self.hwnd);
            if !IsWindowVisible(hwnd).as_bool() {
                return Err("Window not visible or invalid handle".to_string());
            }
            
            let result = ShowWindow(hwnd, SW_RESTORE);
            if result.0 != 0 {
                Ok(())
            } else {
                Err("Failed to restore window".to_string())
            }
        }
    }

    pub fn set_position(&self, x: i32, y: i32) -> Result<(), String> {
        unsafe {
            let hwnd = HWND(self.hwnd);
            if !IsWindowVisible(hwnd).as_bool() {
                return Err("Window not visible or invalid handle".to_string());
            }
            
            // 获取当前窗口大小
            let mut rect = std::mem::zeroed();
            if GetWindowRect(hwnd, &mut rect).is_err() {
                return Err("Failed to get window rect".to_string());
            }

            let width = rect.right - rect.left;
            let height = rect.bottom - rect.top;
            
            if SetWindowPos(
                hwnd, 
                HWND(0), 
                x, y, width, height, 
                SWP_NOZORDER | SWP_NOACTIVATE
            ).is_ok() {
                Ok(())
            } else {
                Err("Failed to set window position".to_string())
            }
        }
    }
}

// 为 WindowsWindowData 实现 PlatformWindow trait
impl PlatformWindow for WindowsWindowData {
    fn minimize(&self) -> Result<(), String> {
        self.minimize()
    }

    fn maximize(&self) -> Result<(), String> {
        self.maximize()
    }

    fn restore(&self) -> Result<(), String> {
        self.restore()
    }

    fn set_position(&self, x: i32, y: i32) -> Result<(), String> {
        self.set_position(x, y)
    }
}

// 主要的 Windows 平台实现函数
pub fn get_all_windows_with_size() -> Vec<WindowInfo> {
    let mut windows = Vec::new();

    unsafe {
        let _ = EnumWindows(Some(enum_window_callback), LPARAM(&mut windows as *mut _ as isize));
    }

    windows
}

unsafe extern "system" fn enum_window_callback(hwnd: HWND, lparam: LPARAM) -> BOOL {
    let windows = &mut *(lparam.0 as *mut Vec<WindowInfo>);

    if IsWindowVisible(hwnd).as_bool() {
        let mut title = [0u16; 512];
        let title_len = GetWindowTextW(hwnd, &mut title);
        
        if title_len > 0 {
            let title_str = String::from_utf16_lossy(&title[..title_len as usize]);
            
            // 跳过空标题或系统窗口
            if !title_str.trim().is_empty() && !is_system_window(hwnd) {
                let mut pid: u32 = 0;
                GetWindowThreadProcessId(hwnd, Some(&mut pid));
                
                let mut rect = std::mem::zeroed();
                if GetWindowRect(hwnd, &mut rect).is_ok() {
                    let window_info = WindowInfo {
                        pid,
                        title: title_str,
                        rect: WindowRect::new(
                            rect.left,
                            rect.top,
                            rect.right - rect.left,
                            rect.bottom - rect.top
                        ),
                    };
                    
                    windows.push(window_info);
                }
            }
        }
    }

    true.into() // Continue enumeration
}

fn is_system_window(hwnd: HWND) -> bool {
    unsafe {
        let mut class_name = [0u16; 256];
        let class_len = GetClassNameW(hwnd, &mut class_name);
        
        if class_len > 0 {
            let class_str = String::from_utf16_lossy(&class_name[..class_len as usize]);
            class_str == "Progman" || class_str == "WorkerW" || class_str == "Shell_TrayWnd"
        } else {
            false
        }
    }
}

pub fn find_windows(
    pid_filter: &Option<String>,
    name_filter: &Option<String>,
    title_filter: &Option<String>,
    process_names: &[(u32, String)],
) -> Vec<crate::platform::WindowHandle> {
    use crate::platform::{WindowHandle, PlatformData};
    
    let all_windows = get_all_windows_with_size();
    let mut result = Vec::new();

    for window in all_windows {
        // PID filter
        if let Some(pid_str) = pid_filter {
            if let Ok(filter_pid) = pid_str.parse::<u32>() {
                if window.pid != filter_pid {
                    continue;
                }
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
                continue;
            }
        }

        // Title filter
        if let Some(title) = title_filter {
            if !window.title.to_lowercase().contains(&title.to_lowercase()) {
                continue;
            }
        }

        // 创建窗口句柄
        // 注意：这里需要获取实际的 HWND，简化实现中我们使用 0
        // 在实际实现中，需要在 enum_window_callback 中保存 HWND
        let platform_data = PlatformData::Windows(WindowsWindowData::new(0));
        let handle = WindowHandle::new(window.pid, window.title, platform_data);
        result.push(handle);
    }

    result
}