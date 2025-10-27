// src/platform/unix.rs
use crate::types::{WindowInfo, WindowRect};
use super::{WindowHandle, PlatformData};
use crate::platform::interface::PlatformWindow;

/// Unix 平台特定的窗口数据
#[derive(Debug, Clone)]
pub struct UnixWindowData;

impl UnixWindowData {
    pub fn new() -> Self {
        Self
    }

    pub fn minimize_impl(&self) -> Result<(), String> {
        Err("Window operations not supported on this platform".to_string())
    }

    pub fn maximize_impl(&self) -> Result<(), String> {
        Err("Window operations not supported on this platform".to_string())
    }

    pub fn restore_impl(&self) -> Result<(), String> {
        Err("Window operations not supported on this platform".to_string())
    }

    pub fn set_position_impl(&self, _x: i32, _y: i32) -> Result<(), String> {
        Err("Window position setting not supported on this platform".to_string())
    }
    
    pub fn set_always_on_top_impl(&self, _on_top: bool) -> Result<(), String> {
        Err("Window always on top operations not supported on this platform".to_string())
    }
    
    pub fn is_always_on_top_impl(&self) -> Result<bool, String> {
        Err("Window always on top detection not supported on this platform".to_string())
    }
}

// 修复这里：避免递归调用
impl PlatformWindow for UnixWindowData {
    fn minimize(&self) -> Result<(), String> {
        self.minimize_impl()
    }

    fn maximize(&self) -> Result<(), String> {
        self.maximize_impl()
    }

    fn restore(&self) -> Result<(), String> {
        self.restore_impl()
    }

    fn set_position(&self, x: i32, y: i32) -> Result<(), String> {
        self.set_position_impl(x, y)
    }
    
    fn set_always_on_top(&self, on_top: bool) -> Result<(), String> {
        self.set_always_on_top_impl(on_top)
    }
    
    fn is_always_on_top(&self) -> Result<bool, String> {
        self.is_always_on_top_impl()
    }
}

pub fn get_all_windows_with_size() -> Vec<WindowInfo> {
    // 在 Unix 系统上返回空向量或使用其他方法
    // 这里可以根据需要集成 x11 或 wayland 支持
    eprintln!("Warning: Window size detection is limited on non-Windows systems");
    Vec::new()
}

pub fn find_windows(
    _pid_filter: &Option<String>,
    _name_filter: &Option<String>,
    _title_filter: &Option<String>,
    _process_names: &[(u32, String)],
) -> Vec<WindowHandle> {
    // 在 Unix 系统上返回空向量
    eprintln!("Warning: Window operations are not supported on this platform");
    Vec::new()
}