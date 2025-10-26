// src/platform/windows.rs
use windows::Win32::Foundation::{HWND, BOOL, LPARAM, RECT};
use windows::Win32::UI::WindowsAndMessaging::{
    EnumWindows, GetWindowThreadProcessId, IsWindowVisible, GetWindowTextW, 
    GetWindowRect, ShowWindow, SW_MINIMIZE, SW_MAXIMIZE, SW_RESTORE,
    SetWindowPos, HWND_TOP, SWP_NOZORDER, SWP_NOSIZE
};
use std::ffi::OsString;
use std::os::windows::ffi::OsStringExt;

use super::{WindowHandle, PlatformData};
use crate::types::{WindowInfo, WindowRect};
use crate::platform::interface::PlatformWindow;  // 添加这行导入

/// Windows 平台特定的窗口数据
#[derive(Debug, Clone)]
pub struct WindowsWindowData {
    pub hwnd: isize, // 使用 isize 而不是 HWND 来避免外部类型依赖
}

impl WindowsWindowData {
    pub fn new(hwnd: HWND) -> Self {
        Self { hwnd: hwnd.0 }
    }

    fn as_hwnd(&self) -> HWND {
        HWND(self.hwnd)
    }

    pub fn minimize(&self) -> Result<(), String> {
        unsafe {
            ShowWindow(self.as_hwnd(), SW_MINIMIZE);
        }
        Ok(())
    }

    pub fn maximize(&self) -> Result<(), String> {
        unsafe {
            ShowWindow(self.as_hwnd(), SW_MAXIMIZE);
        }
        Ok(())
    }

    pub fn restore(&self) -> Result<(), String> {
        unsafe {
            ShowWindow(self.as_hwnd(), SW_RESTORE);
        }
        Ok(())
    }

    pub fn set_position(&self, x: i32, y: i32) -> Result<(), String> {
        unsafe {
            if SetWindowPos(
                self.as_hwnd(),
                HWND_TOP,
                x,
                y,
                0,  // 宽度不变
                0,  // 高度不变
                SWP_NOZORDER | SWP_NOSIZE
            ).is_err() {
                return Err("Failed to set window position".to_string());
            }
        }
        Ok(())
    }
}

// 修复这里：直接使用 PlatformWindow trait，而不是 super::PlatformWindow
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

pub fn get_all_windows_with_size() -> Vec<WindowInfo> {
    let mut windows = Vec::new();
    
    unsafe extern "system" fn enum_windows_proc(hwnd: HWND, lparam: LPARAM) -> BOOL {
        let windows_ptr = lparam.0 as *mut Vec<WindowInfo>;
        
        if IsWindowVisible(hwnd).into() {
            if let Some(window_info) = get_window_info(hwnd) {
                unsafe {
                    (*windows_ptr).push(window_info);
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
    
    windows
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
            if let Some((pid, title)) = get_window_pid_and_title(hwnd) {
                unsafe {
                    let window_data = WindowsWindowData::new(hwnd);
                    (*windows_ptr).push(WindowHandle::new(
                        pid,
                        title,
                        PlatformData::Windows(window_data)
                    ));
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

    apply_window_filters(windows, pid_filter, name_filter, title_filter, process_names)
}

// 辅助函数保持不变
unsafe fn get_window_info(hwnd: HWND) -> Option<WindowInfo> {
    let (pid, title) = get_window_pid_and_title(hwnd)?;
    
    let mut rect = RECT::default();
    if GetWindowRect(hwnd, &mut rect).is_err() {
        return None;
    }
    
    let window_rect = WindowRect::new(
        rect.left,
        rect.top,
        rect.right - rect.left,
        rect.bottom - rect.top
    );
    
    Some(WindowInfo {
        pid,
        title,
        rect: window_rect,
    })
}

unsafe fn get_window_pid_and_title(hwnd: HWND) -> Option<(u32, String)> {
    // Get window title
    let mut title_vec = vec![0u16; 512];
    let title_len = GetWindowTextW(hwnd, &mut title_vec);
    
    if title_len <= 0 {
        return None;
    }
    
    title_vec.truncate(title_len as usize);
    let title_os = OsString::from_wide(&title_vec);
    let title = title_os.to_string_lossy().to_string();
    
    if title.trim().is_empty() {
        return None;
    }
    
    // Get process ID
    let mut pid: u32 = 0;
    GetWindowThreadProcessId(hwnd, Some(&mut pid));
    
    if pid == 0 {
        return None;
    }
    
    Some((pid, title))
}

fn apply_window_filters(
    windows: Vec<WindowHandle>,
    pid_filter: &Option<String>,
    name_filter: &Option<String>,
    title_filter: &Option<String>,
    process_names: &[(u32, String)],
) -> Vec<WindowHandle> {
    windows.into_iter()
        .filter(|window| {
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