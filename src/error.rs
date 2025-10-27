// src/error.rs
use thiserror::Error;

#[derive(Debug, Error)]
pub enum AppError {
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
    
    #[error("Parse error: {0}")]
    Parse(String),
    
    #[error("Window operation failed: {0}")]
    WindowOperation(String),
    
    #[error("No matching windows found")]
    NoMatchingWindows,
    
    #[error("Multiple windows found ({0}). Use --all to modify all matching windows")]
    MultipleWindows(usize),
    
    #[error("Invalid parameter: {0}")]
    InvalidParameter(String),
    
    #[error("Feature not supported: {0}")]
    FeatureNotSupported(String),
    
    #[error("Platform error: {0}")]
    PlatformError(String),
    
    #[error("No windows were modified")]
    NoWindowsModified,

    #[error("Invalid window handle: {0}")]
    InvalidWindowHandle(String),
    
    #[error("Window operation not supported on this platform")]
    PlatformNotSupported,
    
    #[error("Permission denied: {0}")]
    PermissionDenied(String),
}

// 从其他错误类型转换（除了 std::io::Error，它已经用 #[from] 处理了）
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

    pub fn invalid_window_handle(msg: impl Into<String>) -> Self {
        AppError::InvalidWindowHandle(msg.into())
    }
    
    pub fn permission_denied(operation: impl Into<String>) -> Self {
        AppError::PermissionDenied(format!("{} requires elevated privileges", operation.into()))
    }
}

// 结果类型别名
pub type AppResult<T> = Result<T, AppError>;