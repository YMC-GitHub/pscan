// src/platform/unix.rs
use crate::types::{WindowInfo, WindowRect};
use super::WindowHandle;

// Unix 平台的简化实现
pub struct PlatformWindowHandle;

impl PlatformWindowHandle {
    pub fn minimize(&self) -> Result<(), String> {
        Err("Window operations not supported on this platform".to_string())
    }

    pub fn maximize(&self) -> Result<(), String> {
        Err("Window operations not supported on this platform".to_string())
    }

    pub fn restore(&self) -> Result<(), String> {
        Err("Window operations not supported on this platform".to_string())
    }

    // 添加位置设置空实现
    pub fn set_position(&self, _x: i32, _y: i32) -> Result<(), String> {
        Err("Window position setting not supported on this platform".to_string())
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