// src/platform/windows.rs
use windows::Win32::Foundation::{HWND, BOOL, LPARAM, COLORREF};
use windows::Win32::UI::WindowsAndMessaging::{
    EnumWindows, GetWindowTextW, GetWindowThreadProcessId, GetWindowRect, 
    SetWindowPos, ShowWindow, IsWindowVisible, GetClassNameW, GetWindowLongW,
    SW_MINIMIZE, SW_MAXIMIZE, SW_RESTORE, SWP_NOZORDER, SWP_NOACTIVATE, SWP_NOMOVE, SWP_NOSIZE,
    GWL_EXSTYLE, WS_EX_TOPMOST, HWND_TOPMOST, HWND_NOTOPMOST, WS_EX_LAYERED
};
use windows::Win32::UI::WindowsAndMessaging::SetLayeredWindowAttributes;
use windows::Win32::UI::WindowsAndMessaging::LWA_ALPHA;
use windows::Win32::UI::WindowsAndMessaging::SetWindowLongW;
use windows::Win32::UI::WindowsAndMessaging::GetSystemMetrics;  // 新增导入
use windows::Win32::UI::WindowsAndMessaging::SM_CXSCREEN;       // 新增导入
use windows::Win32::UI::WindowsAndMessaging::SM_CYSCREEN;       // 新增导入

use crate::platform::interface::PlatformWindow;
use crate::types::{WindowInfo, WindowRect};
use crate::error::{AppError, AppResult};

/// Windows 平台特定的窗口数据
#[derive(Debug, Clone)]
pub struct WindowsWindowData {
    pub hwnd: isize, // 使用 isize 而不是 HWND 来避免平台特定类型的暴露
}

impl WindowsWindowData {
    pub fn new(hwnd: isize) -> Self {
        Self { hwnd }
    }

    pub fn minimize(&self) -> AppResult<()> {
        unsafe {
            let hwnd = HWND(self.hwnd);
            if !IsWindowVisible(hwnd).as_bool() {
                return Err(AppError::window_operation("Window not visible or invalid handle"));
            }
            
            let result = ShowWindow(hwnd, SW_MINIMIZE);
            if result.0 != 0 {
                Ok(())
            } else {
                Err(AppError::window_operation("Failed to minimize window"))
            }
        }
    }

    pub fn maximize(&self) -> AppResult<()> {
        unsafe {
            let hwnd = HWND(self.hwnd);
            if !IsWindowVisible(hwnd).as_bool() {
                return Err(AppError::window_operation("Window not visible or invalid handle"));
            }
            
            let result = ShowWindow(hwnd, SW_MAXIMIZE);
            if result.0 != 0 {
                Ok(())
            } else {
                Err(AppError::window_operation("Failed to maximize window"))
            }
        }
    }

    pub fn restore(&self) -> AppResult<()> {
        unsafe {
            let hwnd = HWND(self.hwnd);
            if !IsWindowVisible(hwnd).as_bool() {
                return Err(AppError::window_operation("Window not visible or invalid handle"));
            }
            
            let result = ShowWindow(hwnd, SW_RESTORE);
            if result.0 != 0 {
                Ok(())
            } else {
                Err(AppError::window_operation("Failed to restore window"))
            }
        }
    }

    pub fn set_position(&self, x: i32, y: i32) -> AppResult<()> {
        unsafe {
            let hwnd = HWND(self.hwnd);
            if !IsWindowVisible(hwnd).as_bool() {
                return Err(AppError::window_operation("Window not visible or invalid handle"));
            }
            
            // 获取当前窗口大小
            let mut rect = std::mem::zeroed();
            if GetWindowRect(hwnd, &mut rect).is_err() {
                return Err(AppError::platform("Failed to get window rect"));
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
                Err(AppError::window_operation("Failed to set window position"))
            }
        }
    }

    pub fn set_always_on_top(&self, on_top: bool) -> AppResult<()> {
        unsafe {
            let hwnd = HWND(self.hwnd);
            if !IsWindowVisible(hwnd).as_bool() {
                return Err(AppError::window_operation("Window not visible or invalid handle"));
            }
            
            let result = if on_top {
                SetWindowPos(
                    hwnd,
                    HWND_TOPMOST,
                    0, 0, 0, 0,
                    SWP_NOMOVE | SWP_NOSIZE | SWP_NOACTIVATE
                )
            } else {
                SetWindowPos(
                    hwnd,
                    HWND_NOTOPMOST,
                    0, 0, 0, 0,
                    SWP_NOMOVE | SWP_NOSIZE | SWP_NOACTIVATE
                )
            };
            
            if result.is_ok() {
                Ok(())
            } else {
                Err(AppError::window_operation("Failed to set always on top state"))
            }
        }
    }
    
    pub fn is_always_on_top(&self) -> AppResult<bool> {
        unsafe {
            let hwnd = HWND(self.hwnd);
            let ex_style = GetWindowLongW(hwnd, GWL_EXSTYLE);
            
            if ex_style == 0 {
                return Err(AppError::platform("Failed to get window style"));
            }
            
            Ok((ex_style & WS_EX_TOPMOST.0 as i32) != 0)
        }
    }
    
    pub fn set_transparency(&self, opacity: u8) -> AppResult<()> {
        unsafe {
            let hwnd = HWND(self.hwnd);
            if !IsWindowVisible(hwnd).as_bool() {
                return Err(AppError::window_operation("Window not visible or invalid handle"));
            }
            
            // 设置分层窗口样式
            let ex_style = GetWindowLongW(hwnd, GWL_EXSTYLE);
            if ex_style == 0 {
                return Err(AppError::platform("Failed to get window style"));
            }
            
            // 确保窗口有分层样式
            let new_style = ex_style | WS_EX_LAYERED.0 as i32;
            if SetWindowLongW(hwnd, GWL_EXSTYLE, new_style) == 0 {
                return Err(AppError::platform("Failed to set layered window style"));
            }
            
            // 计算透明度值 (0-255)
            let alpha = (opacity as u32 * 255) / 100;
            
            // 设置透明度
            let crkey = COLORREF(0);
            match SetLayeredWindowAttributes(hwnd, crkey, alpha as u8, LWA_ALPHA) {
                Ok(()) => Ok(()),
                Err(e) => Err(AppError::platform(format!("Failed to set window transparency: {}", e)))
            }
        }
    }
    
    pub fn resize(&self, width: i32, height: i32, keep_position: bool, center: bool) -> AppResult<()> {
        unsafe {
            let hwnd = HWND(self.hwnd);
            if !IsWindowVisible(hwnd).as_bool() {
                return Err(AppError::window_operation("Window not visible or invalid handle"));
            }
            
            let mut rect = std::mem::zeroed();
            if GetWindowRect(hwnd, &mut rect).is_err() {
                return Err(AppError::platform("Failed to get window rect"));
            }

            let (x, y) = if keep_position {
                (rect.left, rect.top)
            } else if center {
                // 获取屏幕尺寸并计算居中位置
                let screen_width = GetSystemMetrics(SM_CXSCREEN);
                let screen_height = GetSystemMetrics(SM_CYSCREEN);
                ((screen_width - width) / 2, (screen_height - height) / 2)
            } else {
                (rect.left, rect.top)
            };
            
            if SetWindowPos(
                hwnd, 
                HWND(0), 
                x, y, width, height, 
                SWP_NOZORDER | SWP_NOACTIVATE
            ).is_ok() {
                Ok(())
            } else {
                Err(AppError::window_operation("Failed to resize window"))
            }
        }
    }
}

// 为 WindowsWindowData 实现 PlatformWindow trait
impl PlatformWindow for WindowsWindowData {
    fn minimize(&self) -> AppResult<()> {
        self.minimize()
    }

    fn maximize(&self) -> AppResult<()> {
        self.maximize()
    }

    fn restore(&self) -> AppResult<()> {
        self.restore()
    }

    fn set_position(&self, x: i32, y: i32) -> AppResult<()> {
        self.set_position(x, y)
    }
    
    fn set_always_on_top(&self, on_top: bool) -> AppResult<()> {
        self.set_always_on_top(on_top)
    }
    
    fn is_always_on_top(&self) -> AppResult<bool> {
        self.is_always_on_top()
    }
    
    fn set_transparency(&self, opacity: u8) -> AppResult<()> {
        self.set_transparency(opacity)
    }
    fn resize(&self, width: i32, height: i32, keep_position: bool, center: bool) -> AppResult<()> {
        self.resize(width, height, keep_position, center)
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

// 修改 find_windows 函数来保存实际的 HWND
pub fn find_windows(
    pid_filter: &Option<String>,
    name_filter: &Option<String>,
    title_filter: &Option<String>,
    process_names: &[(u32, String)],
) -> Vec<crate::platform::WindowHandle> {
    use crate::platform::{WindowHandle, PlatformData};
    
    let mut windows_with_handles: Vec<(WindowInfo, isize)> = Vec::new();
    
    // 自定义枚举回调来保存 HWND
    unsafe extern "system" fn enum_window_callback_with_handle(hwnd: HWND, lparam: LPARAM) -> BOOL {
        let windows = &mut *(lparam.0 as *mut Vec<(WindowInfo, isize)>);

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
                        
                        windows.push((window_info, hwnd.0));
                    }
                }
            }
        }

        true.into() // Continue enumeration
    }
    
    unsafe {
        let _ = EnumWindows(Some(enum_window_callback_with_handle), LPARAM(&mut windows_with_handles as *mut _ as isize));
    }
    
    let mut result = Vec::new();

    for (window, hwnd) in windows_with_handles {
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

        // 使用实际的 HWND 创建窗口句柄
        let platform_data = PlatformData::Windows(WindowsWindowData::new(hwnd));
        let handle = WindowHandle::new(window.pid, window.title, platform_data);
        result.push(handle);
    }

    result
}