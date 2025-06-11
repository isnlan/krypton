# Krypton 🔐

<div align="center">

**现代化文件加密/解密工具**

一个使用 Rust 和 egui 构建的跨平台文件加密解密应用程序

[![Rust](https://img.shields.io/badge/language-Rust-orange.svg)](https://www.rust-lang.org/)
[![License](https://img.shields.io/badge/license-MIT-blue.svg)](LICENSE)
[![Build Status](https://img.shields.io/badge/build-passing-brightgreen.svg)]()
[![Version](https://img.shields.io/badge/version-0.1.0-blue.svg)](Cargo.toml)

</div>

---

## ✨ 功能特性

- 🔐 **多种加密算法**：支持 AES-256 和 ChaCha20 加密算法
- 🚀 **多线程处理**：利用并行计算提升加密/解密性能
- 📊 **实时进度跟踪**：可视化显示处理进度和状态
- 🎨 **现代化界面**：基于 egui 的直观用户界面
- 📁 **批量处理**：支持同时处理多个文件
- 🔒 **安全可靠**：采用业界标准的加密算法和安全实践
- 🛡️ **文件名加密**：可选择加密文件名以增强隐私保护
- ⚡ **高性能**：Rust 语言构建，性能卓越
- 🌐 **跨平台**：支持 Windows、macOS 和 Linux

## 🛠️ 技术架构

### 项目结构
```
src/
├── main.rs          # 应用程序入口点
├── models.rs        # 数据模型和类型定义
├── core.rs          # 核心业务逻辑
├── app.rs           # 应用状态管理
├── crypto/          # 加密算法实现
└── ui/              # 用户界面组件
    ├── mod.rs       # UI 模块导出
    ├── panels.rs    # UI 面板组件
    └── dialogs.rs   # 对话框组件
```

### 架构特点

- **高内聚低耦合**：模块化设计，职责清晰
- **事件驱动**：基于事件的 UI 架构，避免借用检查器冲突
- **函数式组件**：无状态的可重用 UI 组件
- **线程安全**：使用 Rust 的安全并发特性

## 🚀 快速开始

### 环境要求

- Rust 1.70.0 或更高版本
- Cargo 包管理器

### 安装步骤

1. **克隆仓库**
   ```bash
   git clone https://github.com/yourusername/krypton.git
   cd krypton
   ```

2. **构建项目**
   ```bash
   cargo build --release
   ```

3. **运行应用**
   ```bash
   cargo run
   ```

### 从源码安装

```bash
cargo install --path .
```

## 📖 使用指南

### 基本操作

1. **设置操作模式**：选择加密或解密模式
2. **选择加密算法**：从 AES-256 或 ChaCha20 中选择
3. **设置密码**：输入强密码保护您的文件
4. **选择文件**：添加需要处理的文件
5. **配置选项**：设置线程数、文件名加密等选项
6. **开始处理**：点击开始按钮进行加密/解密

### 高级设置

- **多线程处理**：调整线程数以优化性能
- **文件名加密**：保护文件名隐私
- **自动删除源文件**：处理后删除原始文件
- **自定义扩展名**：为加密文件设置扩展名

## 🔧 配置选项

| 选项 | 描述 | 默认值 |
|------|------|--------|
| 加密算法 | AES-256 或 ChaCha20 | AES-256 |
| 最大线程数 | 并行处理线程数 | 1 |
| 加密文件名 | 是否加密文件名 | 是 |
| 删除源文件 | 处理后删除原文件 | 是 |
| 文件扩展名 | 加密文件的扩展名 | .enc |

## 🧪 示例代码

项目包含两个演示示例：

### 大文件加密示例
```bash
cargo run --example large_file_encryption
```

### 加密架构演示
```bash
cargo run --example crypto_architecture_demo
```

## 📚 依赖库

- **egui** (0.29.0) - 现代即时模式 GUI 框架
- **eframe** (0.29.0) - egui 的本地应用程序框架
- **aes** (0.8) - AES 加密算法实现
- **aes-gcm** (0.10) - AES-GCM 认证加密
- **chacha20poly1305** (0.10) - ChaCha20-Poly1305 加密
- **argon2** (0.5) - 密码哈希算法
- **rayon** (1.8) - 数据并行处理
- **rfd** (0.15) - 原生文件对话框

## 🤝 贡献指南

我们欢迎所有形式的贡献！请遵循以下步骤：

1. Fork 本仓库
2. 创建特性分支 (`git checkout -b feature/AmazingFeature`)
3. 提交更改 (`git commit -m 'Add some AmazingFeature'`)
4. 推送到分支 (`git push origin feature/AmazingFeature`)
5. 创建 Pull Request

### 开发指南

- 遵循 Rust 官方代码风格
- 添加必要的测试用例
- 更新相关文档
- 确保所有测试通过

```bash
# 运行测试
cargo test

# 检查代码格式
cargo fmt --check

# 运行 Clippy 检查
cargo clippy
```

## 🛡️ 安全性

- 使用业界标准的加密算法（AES-256、ChaCha20）
- 采用 Argon2 进行密码哈希
- 使用认证加密防止数据篡改
- 安全的随机数生成

## 📄 许可证

本项目采用 MIT 许可证 - 查看 [LICENSE](LICENSE) 文件了解详情。

## 🙏 致谢

- [egui](https://github.com/emilk/egui) - 优秀的即时模式 GUI 框架
- [RustCrypto](https://github.com/RustCrypto) - 密码学算法实现
- Rust 社区的支持和贡献

## 📞 联系方式

- 项目主页：[GitHub Repository](https://github.com/yourusername/krypton)
- 问题反馈：[Issues](https://github.com/yourusername/krypton/issues)
- 邮箱：your.email@example.com

---

<div align="center">

**如果这个项目对您有帮助，请给它一个 ⭐️**

Made with ❤️ and Rust

</div>