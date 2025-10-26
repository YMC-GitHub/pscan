use crate::types::{WindowInfo, WindowRect};

pub use crate::platform::{WindowHandle, find_windows, get_all_windows_with_size};

// 保持向后兼容的函数
pub fn get_all_windows() -> Vec<(u32, String)> {
    get_all_windows_with_size()
        .into_iter()
        .map(|window| (window.pid, window.title))
        .collect()
}
// 删除原来的 manipulation 模块，功能已移动到 platform 模块
