// src/platform/unix.rs
use crate::types::{WindowInfo, WindowRect};
use super::{WindowHandle, PlatformData};
use crate::platform::interface::PlatformWindow;  // 添加这行导入

/// Unix 平台特定的窗口数据
#[derive(Debug, Clone)]
pub struct UnixWindowData;

impl UnixWindowData {
    pub fn new() -> Self {
        Self
    }

    pub fn minimize(&self) -> Result<(), String> {
        Err("Window operations not supported on this platform".to_string())
    }

    pub fn maximize(&self) -> Result<(), String> {
        Err("Window operations not supported on this platform".to_string())
    }

    pub fn restore(&self) -> Result<(), String> {
        Err("Window operations not supported on this platform".to_string())
    }

    pub fn set_position(&self, _x: i32, _y: i32) -> Result<(), String> {
        Err("Window position setting not supported on this platform".to_string())
    }
}

// 修复这里：直接使用 PlatformWindow trait
impl PlatformWindow for UnixWindowData {
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