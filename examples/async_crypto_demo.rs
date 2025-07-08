use krypton::core::FileManager;
use krypton::crypto::CryptoEngine;
use krypton::models::{Settings, OperationMode, EncryptionAlgorithm, ProgressInfo, ProgressCallback};
use std::fs;
use std::sync::Arc;
use std::time::Duration;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("🚀 异步加密引擎演示");
    println!("====================");
    
    // 创建测试目录
    let test_dir = "test_async_crypto";
    if std::path::Path::new(test_dir).exists() {
        fs::remove_dir_all(test_dir)?;
    }
    fs::create_dir_all(test_dir)?;
    
    // 创建测试文件
    create_test_files(test_dir)?;
    
    // 演示异步加密
    demo_async_encryption(test_dir)?;
    
    // 演示进度报告
    demo_progress_reporting(test_dir)?;
    
    // 演示操作取消
    demo_operation_cancellation(test_dir)?;
    
    println!("\n🎉 异步演示完成！");
    println!("您可以在 {} 目录下查看生成的文件", test_dir);
    
    Ok(())
}

fn create_test_files(test_dir: &str) -> Result<(), Box<dyn std::error::Error>> {
    println!("\n📁 创建测试文件...");
    
    let test_files = vec![
        ("small_file.txt", "这是一个小文件的内容。".repeat(100)),
        ("medium_file.txt", "这是一个中等大小文件的内容。".repeat(1000)),
        ("large_file.txt", "这是一个大文件的内容。".repeat(10000)),
    ];
    
    for (filename, content) in test_files {
        let file_path = format!("{}/{}", test_dir, filename);
        fs::write(&file_path, content)?;
        println!("  ✅ 创建文件: {}", filename);
    }
    
    Ok(())
}

fn demo_async_encryption(test_dir: &str) -> Result<(), Box<dyn std::error::Error>> {
    println!("\n🔐 演示异步加密...");
    
    // 创建设置
    let settings = Settings {
        password: "async_test_password".to_string(),
        operation_mode: OperationMode::Encrypt,
        encryption_algorithm: EncryptionAlgorithm::AES256,
        max_threads: 2,
        encrypt_filename: false,
        delete_source: false,
        file_extension: "async_enc".to_string(),
    };
    
    // 加载文件
    let files = FileManager::load_files_from_directory(test_dir);
    let mut selected_files = files.clone();
    
    // 选择所有文件进行加密
    for file in &mut selected_files {
        file.selected = true;
    }
    
    // 创建增强的进度回调
    let progress_callback: ProgressCallback = Arc::new(|progress: ProgressInfo| {
        println!("  📊 详细进度报告:");
        println!("    - 文件: {}/{} - {}",
            progress.current_file_index + 1,
            progress.total_files,
            progress.current_file
        );
        println!("    - 当前文件: {:.1}% | 总体: {:.1}%",
            progress.current_file_progress * 100.0,
            progress.overall_progress * 100.0
        );
        println!("    - 速度: {:.2} MB/s", progress.speed_mbps);
        println!("    - 已用时间: {:.1}s", progress.elapsed_time);
        if progress.estimated_remaining > 0.0 {
            println!("    - 预计剩余: {:.1}s", progress.estimated_remaining);
        }
        println!("    - 数据: {:.2} MB / {:.2} MB",
            progress.processed_bytes as f64 / (1024.0 * 1024.0),
            progress.total_bytes as f64 / (1024.0 * 1024.0)
        );
        println!("    ---");
    });
    
    // 启动异步加密
    let start_time = std::time::Instant::now();
    match CryptoEngine::start_operation_async(settings, selected_files, Some(progress_callback)) {
        Ok(handle) => {
            println!("  🚀 异步操作已启动");
            
            // 等待操作完成
            match handle.wait() {
                Ok(_) => {
                    let duration = start_time.elapsed();
                    println!("  ✅ 异步加密完成！耗时: {:?}", duration);
                }
                Err(e) => {
                    println!("  ❌ 异步加密失败: {}", e);
                }
            }
        }
        Err(e) => {
            println!("  ❌ 启动异步操作失败: {}", e);
        }
    }
    
    Ok(())
}

fn demo_progress_reporting(test_dir: &str) -> Result<(), Box<dyn std::error::Error>> {
    println!("\n📊 演示详细进度报告...");
    
    // 创建设置
    let settings = Settings {
        password: "progress_test_password".to_string(),
        operation_mode: OperationMode::Encrypt,
        encryption_algorithm: EncryptionAlgorithm::ChaCha20,
        max_threads: 1, // 使用单线程以便更好地观察进度
        encrypt_filename: false,
        delete_source: false,
        file_extension: "progress_enc".to_string(),
    };
    
    // 加载文件
    let files = FileManager::load_files_from_directory(test_dir);
    let mut selected_files = files.clone();
    
    // 选择所有文件进行加密
    for file in &mut selected_files {
        file.selected = true;
    }
    
    // 创建详细的进度回调
    let progress_callback: ProgressCallback = Arc::new(|progress: ProgressInfo| {
        println!("  📈 详细进度报告:");
        println!("    - 当前文件: {}", progress.current_file);
        println!("    - 文件索引: {}/{}", progress.current_file_index + 1, progress.total_files);
        println!("    - 当前文件进度: {:.1}%", progress.current_file_progress * 100.0);
        println!("    - 总体进度: {:.1}%", progress.overall_progress * 100.0);
        println!("    ---");
    });
    
    // 启动异步加密
    match CryptoEngine::start_operation_async(settings, selected_files, Some(progress_callback)) {
        Ok(handle) => {
            println!("  🚀 带进度报告的异步操作已启动");
            
            // 等待操作完成
            match handle.wait() {
                Ok(_) => {
                    println!("  ✅ 带进度报告的加密完成！");
                }
                Err(e) => {
                    println!("  ❌ 带进度报告的加密失败: {}", e);
                }
            }
        }
        Err(e) => {
            println!("  ❌ 启动带进度报告的操作失败: {}", e);
        }
    }
    
    Ok(())
}

fn demo_operation_cancellation(test_dir: &str) -> Result<(), Box<dyn std::error::Error>> {
    println!("\n🛑 演示操作取消...");
    
    // 创建设置
    let settings = Settings {
        password: "cancel_test_password".to_string(),
        operation_mode: OperationMode::Encrypt,
        encryption_algorithm: EncryptionAlgorithm::AES256,
        max_threads: 1,
        encrypt_filename: false,
        delete_source: false,
        file_extension: "cancel_enc".to_string(),
    };
    
    // 加载文件
    let files = FileManager::load_files_from_directory(test_dir);
    let mut selected_files = files.clone();
    
    // 选择所有文件进行加密
    for file in &mut selected_files {
        file.selected = true;
    }
    
    // 创建进度回调
    let progress_callback: ProgressCallback = Arc::new(|progress: ProgressInfo| {
        println!("  📊 取消演示进度: {}/{} 文件", 
            progress.current_file_index + 1,
            progress.total_files
        );
    });
    
    // 启动异步加密
    match CryptoEngine::start_operation_async(settings, selected_files, Some(progress_callback)) {
        Ok(handle) => {
            println!("  🚀 异步操作已启动，将在1秒后取消");
            
            // 等待1秒后取消操作
            std::thread::sleep(Duration::from_millis(1000));
            handle.stop();
            println!("  🛑 已发送取消信号");
            
            // 等待操作完成（应该被取消）
            match handle.wait() {
                Ok(_) => {
                    println!("  ⚠️  操作意外完成（可能太快了）");
                }
                Err(e) => {
                    println!("  ✅ 操作已取消: {}", e);
                }
            }
        }
        Err(e) => {
            println!("  ❌ 启动取消演示操作失败: {}", e);
        }
    }
    
    Ok(())
}
