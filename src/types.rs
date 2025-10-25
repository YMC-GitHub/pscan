use serde::Serialize;

#[derive(Debug)]
pub struct ProcessInfo {
    pub pid: String,
    pub name: String,
    pub title: String,
    pub memory_usage: u64,
    pub has_window: bool,
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