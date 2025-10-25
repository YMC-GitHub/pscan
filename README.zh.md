# PScan - 进程扫描与分析工具

## 项目描述

PScan 是一个跨平台的进程过滤和分析命令行工具，具有先进的窗口检测功能。它提供了多种输出格式和灵活的过滤选项，帮助开发者和系统管理员快速定位和分析系统进程。

## 项目价值与意义

### 🎯 解决的核心问题
- **快速进程定位**：在大量进程中快速找到目标进程
- **窗口关联分析**：识别哪些进程拥有图形界面窗口
- **多格式输出**：支持表格、JSON、YAML、CSV等多种格式，便于集成到其他工具
- **跨平台兼容**：在Windows和类Unix系统上都能运行

### 💡 应用场景
- 调试应用程序时查找特定进程
- 系统资源监控和分析
- 自动化脚本集成
- 进程管理工具开发

## 功能特性

### 🔍 过滤功能
- **PID过滤**：按进程ID精确过滤
- **名称过滤**：按进程名模糊匹配（不区分大小写）
- **标题过滤**：按窗口标题模糊匹配（不区分大小写）
- **窗口状态过滤**：筛选有窗口或无窗口的进程

### 📊 输出格式
- **表格格式**：易于阅读的表格输出
- **JSON格式**：便于程序解析
- **YAML格式**：结构清晰的配置格式
- **CSV格式**：电子表格兼容
- **简单格式**：简洁的单行输出
- **详细格式**：完整的进程信息展示

### 🖥️ 平台支持
- **Windows**：完整的窗口检测支持
- **Linux/macOS**：基础进程信息支持（窗口检测为占位符）

## 目录结构

```
pscan/
├── Cargo.toml          # 项目配置和依赖管理
├── Cargo.lock          # 依赖版本锁定
├── README.md           # 项目说明文档
└── src/
    ├── main.rs         # 主程序入口
    └── output.rs       # 输出格式处理模块
```

## 安装指南

### 前置要求
- Rust 编程环境（1.70.0 或更高版本）
- Cargo 包管理器

### 安装步骤

1. **从源码安装**
```bash
# 克隆项目
git clone https://github.com/ymc-github/pscan.git
cd pscan

# 编译安装
cargo install --path .
```

2. **从Crates.io安装**（发布后）
```bash
cargo install pscan
```

3. **本地运行**
```bash
# 开发模式运行
cargo run -- --help

# 发布模式运行
cargo run --release -- --help
```

## 使用示例

### 基本使用

```bash
# 查看所有进程（表格格式）
pscan

# 查看详细进程信息
pscan --verbose

# 按进程名过滤
pscan --name "chrome"

# 按窗口标题过滤
pscan --title "Visual Studio"

# 只显示有窗口的进程
pscan --has-window
```

### 过滤功能示例

```bash
# 按PID精确查找
pscan --pid 1234

# 组合过滤：名称包含"code"且有窗口的进程
pscan --name "code" --has-window

# 显示没有窗口的进程
pscan --no-window
```

### 输出格式示例

```bash
# JSON格式输出
pscan --format json

# YAML格式输出
pscan --format yaml

# CSV格式输出（适合导入Excel）
pscan --format csv

# 简单格式输出
pscan --format simple

# 详细格式输出
pscan --format detailed
```

### 高级用法

```bash
# 组合使用过滤和输出格式
pscan --name "firefox" --has-window --format json

# 详细模式下的表格输出
pscan --verbose --format table

# 管道处理：查找Chrome进程并统计数量
pscan --name "chrome" --format simple | wc -l
```

## 输出说明

### 表格列说明
- **PID**: 进程ID
- **Name**: 进程名称
- **Title**: 窗口标题或回退标题
- **Memory**: 内存使用量（MB）
- **Window**: 是否有窗口（仅在详细模式显示）

### 内存单位
- 内存使用量自动转换为MB显示
- 原始内存使用量以字节为单位（在详细输出中显示）

## 开发说明

### 项目架构
- **main.rs**: 负责命令行解析、进程收集和过滤逻辑
- **output.rs**: 负责所有输出格式的渲染和显示

### 扩展开发
要添加新的输出格式，请在 `output.rs` 中的 `OutputFormat` 枚举和 `display_processes` 函数中添加相应处理。

## 故障排除

### 常见问题

1. **窗口检测不工作（非Windows系统）**
   - 目前窗口检测功能仅在Windows系统完整支持
   - 其他系统会显示警告信息

2. **权限问题**
   - 在某些系统上可能需要管理员权限才能访问所有进程信息

3. **进程信息不完整**
   - 某些进程可能无法获取完整的命令行或路径信息

## 作者信息

**开发者**: YeMiancheng  
**邮箱**: ymc.github@gmail.com  
**GitHub**: [ymc-github](https://github.com/ymc-github)

## 版权信息

本项目采用双重许可证：
- MIT License
- Apache License 2.0

您可以根据需要选择其中一种许可证。

## 贡献指南

欢迎提交Issue和Pull Request来帮助改进这个项目！

## 更新日志

### v0.1.0
- 初始版本发布
- 基础进程过滤功能
- 多格式输出支持
- Windows窗口检测功能