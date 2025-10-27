// src/features/mod.rs
mod feature_trait;
mod always_on_top;
mod transparency;
mod position_set;  // 新增位置设置模块

pub use feature_trait::Feature;
pub use always_on_top::AlwaysOnTopFeature;
pub use transparency::TransparencyFeature;
pub use position_set::PositionSetFeature;  // 导出位置设置特性

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
    
    // 条件注册窗口置顶特性
    #[cfg(feature = "always_on_top")]
    register_feature_if_supported(&mut manager, AlwaysOnTopFeature::new(), "always_on_top");
    
    // 条件注册窗口透明度特性
    #[cfg(feature = "transparency")]
    register_feature_if_supported(&mut manager, TransparencyFeature::new(), "transparency");
    
    // 条件注册窗口位置设置特性
    #[cfg(feature = "position_set")]
    register_feature_if_supported(&mut manager, PositionSetFeature::new(), "position_set");
    
    manager
}

/// 获取启用的特性列表（用于调试和信息显示）
pub fn get_enabled_features() -> Vec<&'static str> {
    let mut features = Vec::new();
    
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
    
    features
}