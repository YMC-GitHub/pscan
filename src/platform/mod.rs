// src/platform/mod.rs
use crate::types::{WindowInfo};

#[cfg(windows)]
mod windows;
#[cfg(not(windows))]
mod unix;

#[cfg(windows)]
use windows as platform_impl;
#[cfg(not(windows))]
use unix as platform_impl;

// 统一的平台抽象 trait
#[allow(dead_code)]
pub trait PlatformInterface {
    fn get_all_windows_with_size() -> Vec<WindowInfo>;
    fn find_windows(
        pid_filter: &Option<String>,
        name_filter: &Option<String>,
        title_filter: &Option<String>,
        process_names: &[(u32, String)],
    ) -> Vec<WindowHandle>;
}

// 窗口操作句柄
pub struct WindowHandle {
    pub platform_handle: platform_impl::PlatformWindowHandle,
    pub pid: u32,
    pub title: String,
}

impl WindowHandle {
    pub fn minimize(&self) -> Result<(), String> {
        self.platform_handle.minimize()
    }

    pub fn maximize(&self) -> Result<(), String> {
        self.platform_handle.maximize()
    }

    pub fn restore(&self) -> Result<(), String> {
        self.platform_handle.restore()
    }

    // 添加位置设置方法
    pub fn set_position(&self, x: i32, y: i32) -> Result<(), String> {
        self.platform_handle.set_position(x, y)
    }
}

// 公共接口函数 - 委托给平台实现
pub fn get_all_windows_with_size() -> Vec<WindowInfo> {
    platform_impl::get_all_windows_with_size()
}

pub fn find_windows(
    pid_filter: &Option<String>,
    name_filter: &Option<String>,
    title_filter: &Option<String>,
    process_names: &[(u32, String)],
) -> Vec<WindowHandle> {
    platform_impl::find_windows(pid_filter, name_filter, title_filter, process_names)
}