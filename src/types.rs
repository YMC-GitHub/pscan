use serde::Serialize;

#[derive(Debug)]
pub struct ProcessInfo {
    pub pid: String,
    pub name: String,
    pub title: String,
    pub memory_usage: u64,
    pub has_window: bool,
}

#[derive(Debug, Clone, Serialize)]
pub struct WindowInfo {
    pub pid: u32,
    pub title: String,
    pub rect: WindowRect,
}

#[derive(Debug, Clone, Serialize)]
pub struct WindowRect {
    pub x: i32,
    pub y: i32,
    pub width: i32,
    pub height: i32,
}

impl WindowRect {
    pub fn new(x: i32, y: i32, width: i32, height: i32) -> Self {
        Self { x, y, width, height }
    }
    
    pub fn to_string(&self) -> String {
        format!("{}x{}+{}+{}", self.width, self.height, self.x, self.y)
    }
}

#[derive(Serialize)]
pub struct ProcessOutput {
    pub pid: String,
    pub name: String,
    pub title: String,
    pub memory_usage: u64,
    pub memory_usage_mb: f64,
    pub has_window: bool,
}

#[derive(Serialize)]
pub struct WindowOutput {
    pub pid: String,
    pub name: String,
    pub title: String,
    pub x: i32,
    pub y: i32,
    pub width: i32,
    pub height: i32,
    pub dimensions: String,
}

impl From<&WindowInfo> for WindowOutput {
    fn from(window: &WindowInfo) -> Self {
        WindowOutput {
            pid: window.pid.to_string(),
            name: "".to_string(), // Will be filled later
            title: window.title.clone(),
            x: window.rect.x,
            y: window.rect.y,
            width: window.rect.width,
            height: window.rect.height,
            dimensions: window.rect.to_string(),
        }
    }
}

impl From<&ProcessInfo> for ProcessOutput {
    fn from(process: &ProcessInfo) -> Self {
        ProcessOutput {
            pid: process.pid.clone(),
            name: process.name.clone(),
            title: process.title.clone(),
            memory_usage: process.memory_usage,
            memory_usage_mb: (process.memory_usage as f64) / 1024.0 / 1024.0,
            has_window: process.has_window,
        }
    }
}