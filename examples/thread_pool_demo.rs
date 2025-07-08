use krypton::models::{Settings, FileItem, OperationMode, EncryptionAlgorithm};
use krypton::crypto::CryptoEngine;
use krypton::core::FileManager;
use std::fs;
use std::path::PathBuf;
use std::time::Instant;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    env_logger::init();
    
    println!("🧪 线程池加密引擎演示");
    println!("{}", "=".repeat(50));
    
    // 创建测试目录
    let test_dir = "test_thread_pool";
    if fs::metadata(test_dir).is_ok() {
        fs::remove_dir_all(test_dir)?;
    }
    fs::create_dir_all(test_dir)?;
    
    // 创建多个测试文件
    let test_files = vec![
        "file1.txt",
        "file2.txt", 
        "file3.txt",
        "file4.txt",
        "file5.txt",
    ];
    
    println!("📁 创建测试文件...");
    for (i, filename) in test_files.iter().enumerate() {
        let content = format!("这是测试文件 {} 的内容。\n重复内容: {}\n", i + 1, "测试数据 ".repeat(100));
        let file_path = PathBuf::from(test_dir).join(filename);
        fs::write(&file_path, content)?;
        println!("  ✅ 创建: {}", filename);
    }
    
    // 测试不同线程数的性能
    test_with_thread_count(test_dir, 1)?;
    test_with_thread_count(test_dir, 2)?;
    test_with_thread_count(test_dir, 4)?;
    
    // 清理
    fs::remove_dir_all(test_dir)?;
    println!("\n🧹 清理完成");
    
    Ok(())
}

fn test_with_thread_count(test_dir: &str, thread_count: u32) -> Result<(), Box<dyn std::error::Error>> {
    println!("\n🔧 测试线程数: {}", thread_count);
    println!("{}", "-".repeat(30));
    
    // 创建设置
    let mut settings = Settings {
        operation_mode: OperationMode::Encrypt,
        encryption_algorithm: EncryptionAlgorithm::AES256,
        password: "test_password_123".to_string(),
        max_threads: thread_count,
        encrypt_filename: false,
        delete_source: false,
        file_extension: "enc".to_string(),
    };
    
    // 加载文件
    let mut files = FileManager::load_files_from_directory(test_dir);
    for file in &mut files {
        file.selected = true;
    }
    
    println!("📂 加载了 {} 个文件", files.len());
    
    // 创建加密引擎实例
    let engine = CryptoEngine::from_settings(&settings);
    
    // 测试同步加密
    println!("🔐 开始同步加密...");
    let start_time = Instant::now();
    
    match engine.start_operation(&settings, &files) {
        Ok(_) => {
            let duration = start_time.elapsed();
            println!("  ✅ 同步加密完成，耗时: {:?}", duration);
        }
        Err(e) => {
            println!("  ❌ 同步加密失败: {}", e);
            return Err(e.into());
        }
    }
    
    // 测试异步加密（先删除之前的加密文件）
    let encrypted_files = FileManager::load_encrypted_files_from_directory(test_dir, &settings);
    for file in &encrypted_files {
        if let Err(e) = fs::remove_file(&file.path) {
            println!("  ⚠️  删除加密文件失败: {}", e);
        }
    }
    
    println!("🔐 开始异步加密...");
    let start_time = Instant::now();
    
    // 创建进度回调
    let progress_callback = std::sync::Arc::new(move |progress: krypton::models::ProgressInfo| {
        println!("  📊 进度: {:.1}% - {}", 
                progress.overall_progress * 100.0, 
                progress.current_file);
    });
    
    match engine.start_operation_async(settings.clone(), files.clone(), Some(progress_callback)) {
        Ok(handle) => {
            // 等待完成
            match handle.wait() {
                Ok(_) => {
                    let duration = start_time.elapsed();
                    println!("  ✅ 异步加密完成，耗时: {:?}", duration);
                }
                Err(e) => {
                    println!("  ❌ 异步加密失败: {}", e);
                    return Err(e.into());
                }
            }
        }
        Err(e) => {
            println!("  ❌ 启动异步加密失败: {}", e);
            return Err(e.into());
        }
    }
    
    // 验证加密文件
    let encrypted_files = FileManager::load_encrypted_files_from_directory(test_dir, &settings);
    println!("  📁 生成了 {} 个加密文件", encrypted_files.len());
    
    // 测试解密
    settings.operation_mode = OperationMode::Decrypt;
    let mut decrypt_files = encrypted_files;
    for file in &mut decrypt_files {
        file.selected = true;
    }
    
    println!("🔓 开始解密...");
    let start_time = Instant::now();
    
    match engine.start_operation(&settings, &decrypt_files) {
        Ok(_) => {
            let duration = start_time.elapsed();
            println!("  ✅ 解密完成，耗时: {:?}", duration);
        }
        Err(e) => {
            println!("  ❌ 解密失败: {}", e);
            return Err(e.into());
        }
    }
    
    // 清理加密文件
    for file in &decrypt_files {
        if let Err(e) = fs::remove_file(&file.path) {
            println!("  ⚠️  删除加密文件失败: {}", e);
        }
    }
    
    Ok(())
}
