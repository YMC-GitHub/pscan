// src/error.rs
use std::fmt;

#[derive(Debug)]
pub enum AppError {
    Io(std::io::Error),
    Parse(String),
    WindowOperation(String),
    NoMatchingWindows,
    MultipleWindows(usize),
    InvalidParameter(String),
    FeatureNotSupported(String),
    PlatformError(String),
    NoWindowsModified,
}

impl fmt::Display for AppError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            AppError::Io(e) => write!(f, "IO error: {}", e),
            AppError::Parse(msg) => write!(f, "Parse error: {}", msg),
            AppError::WindowOperation(msg) => write!(f, "Window operation failed: {}", msg),
            AppError::NoMatchingWindows => write!(f, "No matching windows found"),
            AppError::MultipleWindows(count) => write!(
                f, 
                "Multiple windows found ({}). Use --all to modify all matching windows", 
                count
            ),
            AppError::InvalidParameter(msg) => write!(f, "Invalid parameter: {}", msg),
            AppError::FeatureNotSupported(feature) => write!(f, "Feature not supported: {}", feature),
            AppError::PlatformError(msg) => write!(f, "Platform error: {}", msg),
            AppError::NoWindowsModified => write!(f, "No windows were modified"),
        }
    }
}

impl std::error::Error for AppError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            AppError::Io(e) => Some(e),
            _ => None,
        }
    }
}

// 从其他错误类型转换
impl From<std::io::Error> for AppError {
    fn from(err: std::io::Error) -> Self {
        AppError::Io(err)
    }
}

impl From<serde_json::Error> for AppError {
    fn from(err: serde_json::Error) -> Self {
        AppError::Parse(format!("JSON error: {}", err))
    }
}

impl From<serde_yaml::Error> for AppError {
    fn from(err: serde_yaml::Error) -> Self {
        AppError::Parse(format!("YAML error: {}", err))
    }
}

impl From<csv::Error> for AppError {
    fn from(err: csv::Error) -> Self {
        AppError::Parse(format!("CSV error: {}", err))
    }
}

// 便捷构造函数
impl AppError {
    pub fn window_operation(msg: impl Into<String>) -> Self {
        AppError::WindowOperation(msg.into())
    }
    
    pub fn parse(msg: impl Into<String>) -> Self {
        AppError::Parse(msg.into())
    }
    
    pub fn invalid_parameter(msg: impl Into<String>) -> Self {
        AppError::InvalidParameter(msg.into())
    }
    
    pub fn platform(msg: impl Into<String>) -> Self {
        AppError::PlatformError(msg.into())
    }
    
    pub fn feature_not_supported(feature: impl Into<String>) -> Self {
        AppError::FeatureNotSupported(feature.into())
    }
}

// 结果类型别名
pub type AppResult<T> = Result<T, AppError>;