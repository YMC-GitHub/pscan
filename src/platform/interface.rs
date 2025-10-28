// src/platform/interface.rs
use crate::types::WindowInfo;
use crate::error::{AppError, AppResult};

/// 平台窗口句柄的通用接口
pub trait PlatformWindow {
    fn minimize(&self) -> AppResult<()>;
    fn maximize(&self) -> AppResult<()>;
    fn restore(&self) -> AppResult<()>;
    fn set_position(&self, x: i32, y: i32) -> AppResult<()>;
    fn set_always_on_top(&self, on_top: bool) -> AppResult<()>;
    fn is_always_on_top(&self) -> AppResult<bool>;
    fn set_transparency(&self, opacity: u8) -> AppResult<()>;
    fn resize(&self, width: i32, height: i32, keep_position: bool, center: bool) -> AppResult<()>;

}

/// 平台接口 trait
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

/// 统一的窗口句柄
#[derive(Debug, Clone)]
pub struct WindowHandle {
    pub pid: u32,
    pub title: String,
    // 平台特定的句柄数据，但不暴露具体类型
    platform_data: PlatformData,
}

impl WindowHandle {
    pub fn new(pid: u32, title: String, platform_data: PlatformData) -> Self {
        Self { pid, title, platform_data }
    }

    pub fn minimize(&self) -> AppResult<()> {
        self.platform_data.minimize()
    }

    pub fn maximize(&self) -> AppResult<()> {
        self.platform_data.maximize()
    }

    pub fn restore(&self) -> AppResult<()> {
        self.platform_data.restore()
    }

    pub fn set_position(&self, x: i32, y: i32) -> AppResult<()> {
        self.platform_data.set_position(x, y)
    }
    
    pub fn set_always_on_top(&self, on_top: bool) -> AppResult<()> {
        self.platform_data.set_always_on_top(on_top)
    }
    
    pub fn is_always_on_top(&self) -> AppResult<bool> {
        self.platform_data.is_always_on_top()
    }
    
    pub fn set_transparency(&self, opacity: u8) -> AppResult<()> {
        self.platform_data.set_transparency(opacity)
    }
    pub fn resize(&self, width: i32, height: i32, keep_position: bool, center: bool) -> AppResult<()> {
        self.platform_data.resize(width, height, keep_position, center)
    }
}

/// 平台数据枚举，封装不同平台的实现
#[derive(Debug, Clone)]
pub enum PlatformData {
    #[cfg(windows)]
    Windows(crate::platform::windows::WindowsWindowData),
    #[cfg(unix)]
    Unix(crate::platform::unix::UnixWindowData),
}

impl PlatformWindow for PlatformData {
    fn minimize(&self) -> AppResult<()> {
        match self {
            #[cfg(windows)]
            PlatformData::Windows(data) => data.minimize(),
            #[cfg(unix)]
            PlatformData::Unix(data) => data.minimize(),
        }
    }

    fn maximize(&self) -> AppResult<()> {
        match self {
            #[cfg(windows)]
            PlatformData::Windows(data) => data.maximize(),
            #[cfg(unix)]
            PlatformData::Unix(data) => data.maximize(),
        }
    }

    fn restore(&self) -> AppResult<()> {
        match self {
            #[cfg(windows)]
            PlatformData::Windows(data) => data.restore(),
            #[cfg(unix)]
            PlatformData::Unix(data) => data.restore(),
        }
    }

    fn set_position(&self, x: i32, y: i32) -> AppResult<()> {
        match self {
            #[cfg(windows)]
            PlatformData::Windows(data) => data.set_position(x, y),
            #[cfg(unix)]
            PlatformData::Unix(data) => data.set_position(x, y),
        }
    }
    
    fn set_always_on_top(&self, on_top: bool) -> AppResult<()> {
        match self {
            #[cfg(windows)]
            PlatformData::Windows(data) => data.set_always_on_top(on_top),
            #[cfg(unix)]
            PlatformData::Unix(data) => data.set_always_on_top(on_top),
        }
    }
    
    fn is_always_on_top(&self) -> AppResult<bool> {
        match self {
            #[cfg(windows)]
            PlatformData::Windows(data) => data.is_always_on_top(),
            #[cfg(unix)]
            PlatformData::Unix(data) => data.is_always_on_top(),
        }
    }
    
    fn set_transparency(&self, opacity: u8) -> AppResult<()> {
        match self {
            #[cfg(windows)]
            PlatformData::Windows(data) => data.set_transparency(opacity),
            #[cfg(unix)]
            PlatformData::Unix(data) => data.set_transparency(opacity),
        }
    }
    fn resize(&self, width: i32, height: i32, keep_position: bool, center: bool) -> AppResult<()> {
        match self {
            #[cfg(windows)]
            PlatformData::Windows(data) => data.resize(width, height, keep_position, center),
            #[cfg(unix)]
            PlatformData::Unix(data) => data.resize(width, height, keep_position, center),
        }
    }
}