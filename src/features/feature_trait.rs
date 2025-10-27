// src/features/feature_trait.rs
use clap::Command;
use crate::cli::SubCommand;
use crate::error::AppResult;

/// 特性 trait - 所有可插拔功能都需要实现这个 trait
pub trait Feature: Send + Sync {
    /// 特性名称（用于标识）
    fn name(&self) -> &'static str;
    
    /// 特性描述
    fn description(&self) -> &'static str;
    
    /// 构建 CLI 子命令
    fn build_cli(&self, command: Command) -> Command;
    
    /// 解析 CLI 参数并返回子命令（如果匹配）
    fn parse_cli(&self, matches: &clap::ArgMatches) -> Option<SubCommand>;
    
    /// 执行特性功能
    fn execute(&self, subcommand: &SubCommand) -> AppResult<()>;
    
    /// 检查是否支持当前平台
    fn is_supported(&self) -> bool;
}