mod feature_trait;
mod always_on_top;
mod transparency;
mod position_set;
mod window_operations;
mod windows_get;
mod resize;  // 新增

pub use feature_trait::Feature;
pub use always_on_top::AlwaysOnTopFeature;
pub use transparency::TransparencyFeature;
pub use position_set::PositionSetFeature;
pub use window_operations::WindowOperationsFeature;
pub use windows_get::WindowsGetFeature;
pub use resize::ResizeFeature;  // 新增

use std::collections::HashMap;
use crate::error::AppResult;

/// 特性管理器
pub struct FeatureManager {
    features: HashMap<&'static str, Box<dyn Feature>>,
}

impl FeatureManager {
    pub fn new() -> Self {
        Self {
            features: HashMap::new(),
        }
    }
    
    /// 注册特性
    pub fn register_feature(&mut self, feature: Box<dyn Feature>) {
        if feature.is_supported() {
            self.features.insert(feature.name(), feature);
        } else {
            eprintln!("Warning: Feature '{}' is not supported on this platform", feature.name());
        }
    }
    
    /// 获取所有特性
    pub fn get_features(&self) -> Vec<&dyn Feature> {
        self.features.values().map(|f| f.as_ref()).collect()
    }
    
    /// 构建 CLI 命令
    pub fn build_cli(&self, command: clap::Command) -> clap::Command {
        let mut command = command;
        for feature in self.features.values() {
            command = feature.build_cli(command);
        }
        command
    }
    
    /// 解析 CLI 参数
    pub fn parse_cli(&self, matches: &clap::ArgMatches) -> Option<crate::cli::SubCommand> {
        for feature in self.features.values() {
            if let Some(subcommand) = feature.parse_cli(matches) {
                return Some(subcommand);
            }
        }
        None
    }
    
    /// 执行特性命令
    pub fn execute(&self, subcommand: &crate::cli::SubCommand) -> AppResult<()> {
        for feature in self.features.values() {
            if let Err(e) = feature.execute(subcommand) {
                return Err(e);
            }
        }
        Ok(())
    }
}

/// 创建默认特性管理器（包含所有内置特性）
pub fn create_default_manager() -> FeatureManager {
    let mut manager = FeatureManager::new();
    
    // 统一的特性注册函数
    fn register_feature_if_supported<F: Feature + 'static>(
        manager: &mut FeatureManager, 
        feature: F,
        feature_name: &str
    ) {
        if feature.is_supported() {
            manager.register_feature(Box::new(feature));
            if std::env::var("PSCAN_DEBUG_FEATURES").is_ok() {
                println!("Debug: {} feature enabled", feature_name);
            }
        } else {
            eprintln!("Warning: {} feature is not supported on this platform", feature_name);
        }
    }
    
    // 条件注册窗口获取特性（应该总是支持）
    #[cfg(feature = "windows_get")]
    register_feature_if_supported(&mut manager, WindowsGetFeature::new(), "windows_get");

    // 条件注册窗口操作特性
    #[cfg(feature = "window_operations")]
    register_feature_if_supported(&mut manager, WindowOperationsFeature::new(), "window_operations");

    // 条件注册窗口置顶特性
    #[cfg(feature = "always_on_top")]
    register_feature_if_supported(&mut manager, AlwaysOnTopFeature::new(), "always_on_top");
    
    // 条件注册窗口透明度特性
    #[cfg(feature = "transparency")]
    register_feature_if_supported(&mut manager, TransparencyFeature::new(), "transparency");
    
    // 条件注册窗口位置设置特性
    #[cfg(feature = "position_set")]
    register_feature_if_supported(&mut manager, PositionSetFeature::new(), "position_set");
    
    // 条件注册窗口调整大小特性
    #[cfg(feature = "resize")]
    register_feature_if_supported(&mut manager, ResizeFeature::new(), "resize");
    
    manager
}

/// 获取启用的特性列表（用于调试和信息显示）
pub fn get_enabled_features() -> Vec<&'static str> {
    let mut features = Vec::new();
    
    #[cfg(feature = "windows_get")]
    {
        features.push("windows_get");
    }
    
    #[cfg(feature = "window_operations")]
    {
        features.push("window_operations");
    }
    
    #[cfg(feature = "always_on_top")]
    {
        features.push("always_on_top");
    }
    
    #[cfg(feature = "transparency")]
    {
        features.push("transparency");
    }
    
    #[cfg(feature = "position_set")]
    {
        features.push("position_set");
    }
    
    #[cfg(feature = "resize")]
    {
        features.push("resize");
    }
    
    features
}