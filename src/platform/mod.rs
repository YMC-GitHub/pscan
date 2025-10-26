// src/platform/mod.rs
mod interface;
#[cfg(windows)]
mod windows;
#[cfg(unix)]
mod unix;

pub use interface::{WindowHandle, PlatformData};

// 平台特定的实现函数
#[cfg(windows)]
pub fn get_all_windows_with_size() -> Vec<crate::types::WindowInfo> {
    windows::get_all_windows_with_size()
}

#[cfg(windows)]
pub fn find_windows(
    pid_filter: &Option<String>,
    name_filter: &Option<String>,
    title_filter: &Option<String>,
    process_names: &[(u32, String)],
) -> Vec<WindowHandle> {
    windows::find_windows(pid_filter, name_filter, title_filter, process_names)
}

#[cfg(unix)]
pub fn get_all_windows_with_size() -> Vec<crate::types::WindowInfo> {
    unix::get_all_windows_with_size()
}

#[cfg(unix)]
pub fn find_windows(
    pid_filter: &Option<String>,
    name_filter: &Option<String>,
    title_filter: &Option<String>,
    process_names: &[(u32, String)],
) -> Vec<WindowHandle> {
    unix::find_windows(pid_filter, name_filter, title_filter, process_names)
}