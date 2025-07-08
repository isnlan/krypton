# Krypton 中文字体支持说明

## 概述

Krypton 现已支持中文字体显示，可以正确显示中文文件名和路径。应用程序会自动检测并加载系统中文字体，确保中文字符的正确显示。界面语言为英文，但完全支持中文文件处理。

## 中文字体支持

### 自动字体检测

应用程序启动时会自动检测并加载系统中文字体，支持的字体路径包括：

#### macOS 系统字体
- `/System/Library/Fonts/PingFang.ttc` - 苹方字体
- `/System/Library/Fonts/Hiragino Sans GB.ttc` - 冬青黑体
- `/System/Library/Fonts/STHeiti Light.ttc` - 华文黑体

#### Windows 系统字体
- `C:\Windows\Fonts\msyh.ttc` - 微软雅黑
- `C:\Windows\Fonts\simsun.ttc` - 宋体
- `C:\Windows\Fonts\simhei.ttf` - 黑体

#### Linux 系统字体
- `/usr/share/fonts/truetype/droid/DroidSansFallbackFull.ttf`
- `/usr/share/fonts/truetype/wqy/wqy-microhei.ttc` - 文泉驿微米黑
- `/usr/share/fonts/opentype/noto/NotoSansCJK-Regular.ttc` - Noto Sans CJK

### 字体加载状态

应用程序启动时会在控制台输出字体加载状态：
- 成功加载：`成功加载系统字体: [字体路径]`
- 未找到字体：`未找到系统中文字体，将使用内置支持`

## 中文字符支持

### 支持的中文功能

应用程序完全支持中文字符的显示和处理：

#### 文件处理
- **中文文件名**: 完全支持包含中文字符的文件名
- **中文路径**: 支持包含中文字符的文件路径
- **中文目录**: 可以选择和处理包含中文字符的目录
- **文件列表显示**: 正确显示中文文件名

#### 界面显示
- **英文界面**: 界面语言为英文，保持国际化标准
- **中文字体**: 自动加载系统中文字体，确保中文字符正确显示
- **混合文本**: 支持英文界面中显示中文文件名和路径

#### 输入支持
- **密码输入**: 支持中文字符作为密码
- **路径输入**: 支持手动输入包含中文的文件路径
- **文件扩展名**: 支持中文字符的文件扩展名

## 错误消息中文化

### 加密引擎错误消息
- **密码验证**: "密码不能为空"
- **文件选择**: "未选择任何文件"
- **文件操作**: 
  - "无法打开文件 'filename': error"
  - "无法创建输出文件: error"
  - "加密文件 'filename' 失败: error"
  - "解密文件 'filename' 失败: error"
  - "删除源文件失败: error"

### 算法信息
- **算法信息格式**: "算法: AES-256-GCM, 分块大小: 1 MB"

## 技术实现

### 字体加载机制

```rust
fn load_chinese_fonts(ctx: &egui::Context) {
    let mut fonts = egui::FontDefinitions::default();
    
    // 尝试加载系统中文字体
    let chinese_font_data = load_system_chinese_font();
    
    if let Some(font_data) = chinese_font_data {
        fonts.font_data.insert(
            "chinese_font".to_owned(),
            font_data,
        );
        
        // 设置为默认字体的第一优先级
        fonts
            .families
            .entry(egui::FontFamily::Proportional)
            .or_default()
            .insert(0, "chinese_font".to_owned());
    }

    ctx.set_fonts(fonts);
}
```

### 字体优先级

1. **系统中文字体** - 优先使用系统安装的中文字体
2. **内置字体回退** - 如果没有找到系统字体，使用 egui 内置字体支持

## 使用说明

### 启动应用程序

```bash
cargo run
```

应用程序启动后会自动：
1. 检测系统中文字体
2. 加载合适的字体文件
3. 配置字体优先级
4. 显示完全中文化的界面

### 验证中文显示

启动后检查以下元素是否正确显示中文：
- [ ] 窗口标题显示为 "Krypton - 文件加密解密工具"
- [ ] 所有按钮和标签显示中文文本
- [ ] 输入框中的中文字符正确显示
- [ ] 对话框标题和内容为中文
- [ ] 错误消息为中文

## 故障排除

### 中文字符显示为方块或乱码

1. **检查字体加载状态**：查看控制台输出，确认是否成功加载中文字体
2. **手动安装字体**：如果系统没有中文字体，请安装相应的中文字体包
3. **重启应用程序**：字体安装后重新启动应用程序

### 部分中文字符无法显示

1. **字体覆盖范围**：某些字体可能不包含所有中文字符
2. **字体文件损坏**：尝试重新安装系统字体
3. **编码问题**：确保源代码文件使用 UTF-8 编码

## 开发说明

### 添加新的中文文本

在代码中添加新的中文文本时：

1. 直接使用中文字符串：`ui.label("新的中文文本");`
2. 确保源文件使用 UTF-8 编码
3. 测试在不同系统上的显示效果

### 扩展字体支持

如需添加更多字体路径：

```rust
let font_paths = vec![
    // 添加新的字体路径
    "/path/to/new/chinese/font.ttf",
    // ... 现有路径
];
```

## 版本信息

- **支持版本**: Krypton v0.1.0+
- **字体引擎**: egui 0.29.0
- **字符编码**: UTF-8
- **支持语言**: 简体中文、繁体中文、英文

---

如有任何中文显示问题，请提交 Issue 并包含：
1. 操作系统版本
2. 控制台输出的字体加载信息
3. 问题截图
4. 系统已安装的中文字体列表
