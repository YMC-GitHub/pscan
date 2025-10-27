// src/platform/interface.rs
use crate::types::WindowInfo;

/// 平台窗口句柄的通用接口
pub trait PlatformWindow {
    fn minimize(&self) -> Result<(), String>;
    fn maximize(&self) -> Result<(), String>;
    fn restore(&self) -> Result<(), String>;
    fn set_position(&self, x: i32, y: i32) -> Result<(), String>;
    fn set_always_on_top(&self, on_top: bool) -> Result<(), String>;
    fn is_always_on_top(&self) -> Result<bool, String>;
    fn set_transparency(&self, opacity: u8) -> Result<(), String>;  // 新增透明度方法
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

    pub fn minimize(&self) -> Result<(), String> {
        self.platform_data.minimize()
    }

    pub fn maximize(&self) -> Result<(), String> {
        self.platform_data.maximize()
    }

    pub fn restore(&self) -> Result<(), String> {
        self.platform_data.restore()
    }

    pub fn set_position(&self, x: i32, y: i32) -> Result<(), String> {
        self.platform_data.set_position(x, y)
    }
    
    pub fn set_always_on_top(&self, on_top: bool) -> Result<(), String> {
        self.platform_data.set_always_on_top(on_top)
    }
    
    pub fn is_always_on_top(&self) -> Result<bool, String> {
        self.platform_data.is_always_on_top()
    }
    
    pub fn set_transparency(&self, opacity: u8) -> Result<(), String> {
        self.platform_data.set_transparency(opacity)
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
    fn minimize(&self) -> Result<(), String> {
        match self {
            #[cfg(windows)]
            PlatformData::Windows(data) => data.minimize(),
            #[cfg(unix)]
            PlatformData::Unix(data) => data.minimize(),
        }
    }

    fn maximize(&self) -> Result<(), String> {
        match self {
            #[cfg(windows)]
            PlatformData::Windows(data) => data.maximize(),
            #[cfg(unix)]
            PlatformData::Unix(data) => data.maximize(),
        }
    }

    fn restore(&self) -> Result<(), String> {
        match self {
            #[cfg(windows)]
            PlatformData::Windows(data) => data.restore(),
            #[cfg(unix)]
            PlatformData::Unix(data) => data.restore(),
        }
    }

    fn set_position(&self, x: i32, y: i32) -> Result<(), String> {
        match self {
            #[cfg(windows)]
            PlatformData::Windows(data) => data.set_position(x, y),
            #[cfg(unix)]
            PlatformData::Unix(data) => data.set_position(x, y),
        }
    }
    
    fn set_always_on_top(&self, on_top: bool) -> Result<(), String> {
        match self {
            #[cfg(windows)]
            PlatformData::Windows(data) => data.set_always_on_top(on_top),
            #[cfg(unix)]
            PlatformData::Unix(data) => data.set_always_on_top(on_top),
        }
    }
    
    fn is_always_on_top(&self) -> Result<bool, String> {
        match self {
            #[cfg(windows)]
            PlatformData::Windows(data) => data.is_always_on_top(),
            #[cfg(unix)]
            PlatformData::Unix(data) => data.is_always_on_top(),
        }
    }
    
    fn set_transparency(&self, opacity: u8) -> Result<(), String> {
        match self {
            #[cfg(windows)]
            PlatformData::Windows(data) => data.set_transparency(opacity),
            #[cfg(unix)]
            PlatformData::Unix(data) => data.set_transparency(opacity),
        }
    }
}