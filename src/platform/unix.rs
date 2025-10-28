// src/platform/unix.rs
use crate::types::{WindowInfo, WindowRect};
use super::{WindowHandle, PlatformData};
use crate::platform::interface::PlatformWindow;
use crate::error::{AppError, AppResult};

/// Unix 平台特定的窗口数据
#[derive(Debug, Clone)]
pub struct UnixWindowData;

impl UnixWindowData {
    pub fn new() -> Self {
        Self
    }

    pub fn minimize_impl(&self) -> AppResult<()> {
        Err(AppError::feature_not_supported("Window operations"))
    }

    pub fn maximize_impl(&self) -> AppResult<()> {
        Err(AppError::feature_not_supported("Window operations"))
    }

    pub fn restore_impl(&self) -> AppResult<()> {
        Err(AppError::feature_not_supported("Window operations"))
    }

    pub fn set_position_impl(&self, _x: i32, _y: i32) -> AppResult<()> {
        Err(AppError::feature_not_supported("Window position setting"))
    }
    
    pub fn set_always_on_top_impl(&self, _on_top: bool) -> AppResult<()> {
        Err(AppError::feature_not_supported("Window always on top operations"))
    }
    
    pub fn is_always_on_top_impl(&self) -> AppResult<bool> {
        Err(AppError::feature_not_supported("Window always on top detection"))
    }
    
    pub fn set_transparency_impl(&self, _opacity: u8) -> AppResult<()> {
        Err(AppError::feature_not_supported("Window transparency operations"))
    }

    pub fn resize_impl(&self, _width: i32, _height: i32, _keep_position: bool, _center: bool) -> AppResult<()> {
        Err(AppError::feature_not_supported("Window resizing"))
    }
}

// 修复这里：避免递归调用
impl PlatformWindow for UnixWindowData {
    fn minimize(&self) -> AppResult<()> {
        self.minimize_impl()
    }

    fn maximize(&self) -> AppResult<()> {
        self.maximize_impl()
    }

    fn restore(&self) -> AppResult<()> {
        self.restore_impl()
    }

    fn set_position(&self, x: i32, y: i32) -> AppResult<()> {
        self.set_position_impl(x, y)
    }
    
    fn set_always_on_top(&self, on_top: bool) -> AppResult<()> {
        self.set_always_on_top_impl(on_top)
    }
    
    fn is_always_on_top(&self) -> AppResult<bool> {
        self.is_always_on_top_impl()
    }
    
    fn set_transparency(&self, opacity: u8) -> AppResult<()> {
        self.set_transparency_impl(opacity)
    }

    fn resize(&self, width: i32, height: i32, keep_position: bool, center: bool) -> AppResult<()> {
        self.resize_impl(width, height, keep_position, center)
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