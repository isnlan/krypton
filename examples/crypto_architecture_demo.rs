use krypton::core::FileManager;
use krypton::crypto::{CryptoEngine, create_crypto_provider, encrypt_stream, decrypt_stream};
use krypton::crypto::traits::CryptoProvider;
use krypton::models::{FileItem, Settings, OperationMode, EncryptionAlgorithm};
use std::fs;
use std::io::Cursor;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("🏗️  新架构演示：策略模式加密系统");
    println!("=====================================");
    
    // 1. 演示策略模式的使用
    demo_strategy_pattern()?;
    
    // 2. 演示工厂模式的使用
    demo_factory_pattern()?;
    
    // 3. 演示完整的文件加密流程
    demo_full_encryption_workflow()?;
    
    println!("\n🎉 所有演示完成！");
    Ok(())
}

/// 演示策略模式：不同的加密算法可以互换使用
fn demo_strategy_pattern() -> Result<(), Box<dyn std::error::Error>> {
    println!("\n📦 策略模式演示");
    println!("----------------");
    
    let test_data = "Hello, World! This is a test message for encryption.";
    let password = "test_password_123";
    
    // 测试不同的加密算法
    let algorithms = vec![
        EncryptionAlgorithm::AES256,
        EncryptionAlgorithm::ChaCha20,
    ];
    
    for algorithm in algorithms {
        println!("\n🔐 测试算法: {:?}", algorithm);
        
        // 创建加密提供者
        let provider = create_crypto_provider(&algorithm);
        println!("  - 算法名称: {}", provider.algorithm_name());
        println!("  - 分块大小: {} KB", provider.chunk_size() / 1024);
        
        // 加密数据
        let mut input = Cursor::new(test_data.as_bytes());
        let mut encrypted = Vec::new();
        
        provider.encrypt_stream(password, &mut input, &mut encrypted)?;
        println!("  - 原始数据: {} 字节", test_data.len());
        println!("  - 加密数据: {} 字节", encrypted.len());
        
        // 解密数据
        let mut encrypted_input = Cursor::new(&encrypted);
        let mut decrypted = Vec::new();
        
        provider.decrypt_stream(password, &mut encrypted_input, &mut decrypted)?;
        let decrypted_text = String::from_utf8(decrypted)?;
        
        println!("  - 解密成功: {}", decrypted_text == test_data);
    }
    
    Ok(())
}

/// 演示工厂模式：通过工厂函数创建不同的加密实现
fn demo_factory_pattern() -> Result<(), Box<dyn std::error::Error>> {
    println!("\n🏭 工厂模式演示");
    println!("----------------");
    
    let test_data = "Factory pattern test data";
    let password = "factory_password";
    
    // 直接使用工厂函数进行加密/解密
    let mut input = Cursor::new(test_data.as_bytes());
    let mut encrypted = Vec::new();
    
    // 使用AES加密
    encrypt_stream(&EncryptionAlgorithm::AES256, password, &mut input, &mut encrypted)?;
    println!("✅ AES加密完成: {} -> {} 字节", test_data.len(), encrypted.len());
    
    // 使用相同算法解密
    let mut encrypted_input = Cursor::new(&encrypted);
    let mut decrypted = Vec::new();
    
    decrypt_stream(&EncryptionAlgorithm::AES256, password, &mut encrypted_input, &mut decrypted)?;
    let decrypted_text = String::from_utf8(decrypted)?;
    
    println!("✅ AES解密完成: {}", decrypted_text == test_data);
    
    Ok(())
}

/// 演示完整的文件加密工作流程
fn demo_full_encryption_workflow() -> Result<(), Box<dyn std::error::Error>> {
    println!("\n🔄 完整工作流程演示");
    println!("------------------");
    
    // 创建测试目录和文件
    let test_dir = "./test_architecture";
    fs::create_dir_all(test_dir)?;
    
    let long_content = "这是第三个测试文件，包含更多数据: ".to_string() + &"data ".repeat(100);
    let test_files = vec![
        ("test1.txt", "这是第一个测试文件的内容"),
        ("test2.txt", "这是第二个测试文件的内容"),
        ("test3.txt", &long_content),
    ];
    
    // 创建测试文件
    for (filename, content) in &test_files {
        let file_path = format!("{}/{}", test_dir, filename);
        fs::write(&file_path, content)?;
        println!("📄 创建文件: {} ({} 字节)", filename, content.len());
    }
    
    // 测试不同的加密算法
    let algorithms = vec![
        EncryptionAlgorithm::AES256,
        EncryptionAlgorithm::ChaCha20,
    ];
    
    for (i, algorithm) in algorithms.iter().enumerate() {
        println!("\n🔐 测试算法 {}: {:?}", i + 1, algorithm);
        
        // 创建设置
        let mut settings = Settings::default();
        settings.password = "workflow_test_password".to_string();
        settings.operation_mode = OperationMode::Encrypt;
        settings.encryption_algorithm = algorithm.clone();
        settings.max_threads = 2;
        settings.encrypt_filename = false;
        settings.delete_source = false;
        settings.file_extension = format!("enc{}", i + 1);
        
        // 显示算法信息
        let info = CryptoEngine::get_algorithm_info(&settings);
        println!("  📊 {}", info);
        
        // 加载文件
        let files = FileManager::load_files_from_directory(test_dir);
        let mut selected_files = files.clone();
        
        // 选择所有文件进行加密
        for file in &mut selected_files {
            file.selected = true;
        }
        
        // 执行加密
        let start_time = std::time::Instant::now();
        match CryptoEngine::start_operation(&settings, &selected_files) {
            Ok(_) => {
                let duration = start_time.elapsed();
                println!("  ✅ 加密完成，耗时: {:?}", duration);
                
                // 统计加密文件
                let encrypted_files = FileManager::load_encrypted_files_from_directory(test_dir, &settings);
                println!("  📁 生成 {} 个加密文件", encrypted_files.len());
                
                // 测试解密
                settings.operation_mode = OperationMode::Decrypt;
                let mut decrypt_files = encrypted_files;
                for file in &mut decrypt_files {
                    file.selected = true;
                }
                
                let decrypt_start = std::time::Instant::now();
                match CryptoEngine::start_operation(&settings, &decrypt_files) {
                    Ok(_) => {
                        let decrypt_duration = decrypt_start.elapsed();
                        println!("  ✅ 解密完成，耗时: {:?}", decrypt_duration);
                    }
                    Err(e) => {
                        println!("  ❌ 解密失败: {}", e);
                    }
                }
            }
            Err(e) => {
                println!("  ❌ 加密失败: {}", e);
            }
        }
    }
    
    // 显示最终文件统计
    let final_files = FileManager::load_files_from_directory(test_dir);
    println!("\n📋 最终文件统计:");
    println!("  - 总文件数: {}", final_files.len());
    
    let mut total_size = 0;
    for file in &final_files {
        if let Ok(metadata) = fs::metadata(&file.path) {
            total_size += metadata.len();
            println!("    {} ({} 字节)", file.name, metadata.len());
        }
    }
    println!("  - 总大小: {} 字节", total_size);
    
    println!("\n💡 架构优势:");
    println!("  - ✅ 模块化设计，易于维护");
    println!("  - ✅ 策略模式，算法可插拔");
    println!("  - ✅ 工厂模式，创建过程统一");
    println!("  - ✅ 错误处理统一且详细");
    println!("  - ✅ 代码复用性高");
    
    Ok(())
} 