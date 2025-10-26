// src/window.rs
// 简化的窗口模块，只提供向后兼容的函数
use crate::platform;

/// 获取所有窗口的PID和标题（保持向后兼容）
pub fn get_all_windows() -> Vec<(u32, String)> {
    platform::get_all_windows_with_size()
        .into_iter()
        .map(|window| (window.pid, window.title))
        .collect()
}

/// 重新导出平台接口的主要功能
#[allow(unused_imports)]
pub use platform::{
    get_all_windows_with_size,
    find_windows,
    WindowHandle,
};