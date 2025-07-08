mod models;
mod core;
mod crypto;
mod ui;
mod app;
mod progress;

use app::KryptonApp;
use eframe::egui;

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

        // 同时设置到等宽字体
        fonts
            .families
            .entry(egui::FontFamily::Monospace)
            .or_default()
            .insert(0, "chinese_font".to_owned());
    } else {
        // 如果没有找到系统字体，使用内置的中文字体支持
        setup_builtin_chinese_support(&mut fonts);
    }

    ctx.set_fonts(fonts);
}

fn load_system_chinese_font() -> Option<egui::FontData> {
    // 尝试加载系统中文字体
    let font_paths = vec![
        // macOS 系统字体
        "/System/Library/Fonts/PingFang.ttc",
        "/System/Library/Fonts/Hiragino Sans GB.ttc",
        "/System/Library/Fonts/STHeiti Light.ttc",
        // Windows 系统字体
        "C:\\Windows\\Fonts\\msyh.ttc",
        "C:\\Windows\\Fonts\\simsun.ttc",
        "C:\\Windows\\Fonts\\simhei.ttf",
        // Linux 系统字体
        "/usr/share/fonts/truetype/droid/DroidSansFallbackFull.ttf",
        "/usr/share/fonts/truetype/wqy/wqy-microhei.ttc",
        "/usr/share/fonts/opentype/noto/NotoSansCJK-Regular.ttc",
    ];

    for path in font_paths {
        if let Ok(font_data) = std::fs::read(path) {
            println!("成功加载系统字体: {}", path);
            return Some(egui::FontData::from_owned(font_data));
        }
    }

    println!("未找到系统中文字体，将使用内置支持");
    None
}

fn setup_builtin_chinese_support(fonts: &mut egui::FontDefinitions) {
    // 确保默认字体支持基本的中文字符
    // egui 的默认字体已经包含了一些中文字符支持
    // 我们可以调整字体回退顺序来改善中文显示

    // 添加 emoji 字体支持（可能包含一些中文字符）
    if let Some(_emoji_font) = fonts.font_data.get("emoji-icon-font") {
        fonts
            .families
            .entry(egui::FontFamily::Proportional)
            .or_default()
            .push("emoji-icon-font".to_owned());
    }
}


fn main() -> Result<(), eframe::Error> {
    env_logger::init();
    
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([1000.0, 700.0])
            .with_title("Krypton - File Encryption Tool"),
        ..Default::default()
    };

    eframe::run_native(
        "Krypton - File Encryption Tool",
        options,
        Box::new(|cc| {
            load_chinese_fonts(&cc.egui_ctx);

            Ok(Box::new(KryptonApp::new()))
        }),
    )
}
