use krypton::core::FileManager;
use krypton::crypto::CryptoEngine;
use krypton::models::{Settings, OperationMode, EncryptionAlgorithm};
use std::fs;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("大文件加密演示");
    println!("================");
    
    // 创建演示设置
    let mut settings = Settings::default();
    settings.password = "my_secure_password_123".to_string();
    settings.operation_mode = OperationMode::Encrypt;
    settings.encryption_algorithm = EncryptionAlgorithm::AES256;
    settings.max_threads = 4; // 启用多线程处理
    settings.encrypt_filename = false; // 保持原文件名
    settings.delete_source = false; // 保留源文件
    settings.file_extension = "enc".to_string();
    
    println!("设置信息:");
    println!("- 加密算法: {:?}", settings.encryption_algorithm);
    println!("- 最大线程数: {}", settings.max_threads);
    println!("- 密码长度: {} 字符", settings.password.len());
    println!("- 文件扩展名: .{}", settings.file_extension);
    println!();
    
    // 创建测试目录和大文件
    let test_dir = "./test_files";
    fs::create_dir_all(test_dir)?;
    
    // 创建一个测试大文件（1MB 随机数据）
    let test_file_path = format!("{}/large_test_file.txt", test_dir);
    if !std::path::Path::new(&test_file_path).exists() {
        println!("创建测试文件: {}", test_file_path);
        let test_data = "A".repeat(1024 * 1024); // 1MB 的 'A' 字符
        fs::write(&test_file_path, test_data)?;
        println!("已创建 1MB 测试文件");
    }
    
    // 加载文件
    let files = FileManager::load_files_from_directory(test_dir);
    println!("找到 {} 个文件:", files.len());
    for file in &files {
        println!("- {}", file.name);
    }
    println!();
    
    // 选择要加密的文件
    let mut selected_files = files;
    for file in &mut selected_files {
        if file.name == "large_test_file.txt" {
            file.selected = true;
            println!("已选择文件进行加密: {}", file.name);
        }
    }
    
    // 执行加密
    println!("开始加密...");
    let start_time = std::time::Instant::now();
    
    match CryptoEngine::start_operation(&settings, &selected_files) {
        Ok(_) => {
            let duration = start_time.elapsed();
            println!("✅ 加密完成！耗时: {:?}", duration);
            
            // 显示加密后的文件
            let encrypted_files = FileManager::load_encrypted_files_from_directory(test_dir, &settings);
            println!("加密后的文件:");
            for file in &encrypted_files {
                println!("- {}", file.name);
                
                // 显示文件大小信息
                if let Ok(metadata) = fs::metadata(&file.path) {
                    println!("  大小: {} 字节", metadata.len());
                }
            }
            
            // 演示解密
            println!("\n开始解密...");
            settings.operation_mode = OperationMode::Decrypt;
            
            // 选择加密文件进行解密
            let mut decrypt_files = encrypted_files;
            for file in &mut decrypt_files {
                file.selected = true;
            }
            
            let decrypt_start = std::time::Instant::now();
            match CryptoEngine::start_operation(&settings, &decrypt_files) {
                Ok(_) => {
                    let decrypt_duration = decrypt_start.elapsed();
                    println!("✅ 解密完成！耗时: {:?}", decrypt_duration);
                    
                    // 验证解密后的文件
                    let final_files = FileManager::load_files_from_directory(test_dir);
                    println!("最终文件列表:");
                    for file in &final_files {
                        println!("- {}", file.name);
                        if let Ok(metadata) = fs::metadata(&file.path) {
                            println!("  大小: {} 字节", metadata.len());
                        }
                    }
                }
                Err(e) => {
                    println!("❌ 解密失败: {}", e);
                }
            }
        }
        Err(e) => {
            println!("❌ 加密失败: {}", e);
        }
    }
    
    println!("\n演示完成!");
    println!("您可以在 {} 目录下查看生成的文件", test_dir);
    
    Ok(())
} 