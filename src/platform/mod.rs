// src/platform/mod.rs
// 移除对上层模块的依赖，只使用相对路径
mod interface;
#[cfg(windows)]
mod windows;
#[cfg(unix)]
mod unix;

pub use interface::{WindowHandle, PlatformData};

// 平台实现选择
#[cfg(windows)]
use windows as platform_impl;
#[cfg(unix)]
use unix as platform_impl;

// 公共接口函数 - 委托给平台实现
pub fn get_all_windows_with_size() -> Vec<crate::types::WindowInfo> {
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