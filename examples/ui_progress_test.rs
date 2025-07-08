use krypton::core::FileManager;
use krypton::crypto::CryptoEngine;
use krypton::models::{Settings, OperationMode, EncryptionAlgorithm, ProgressInfo};
use std::fs;
use std::time::Duration;
use std::thread;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("🧪 UI进度显示测试");
    println!("==================");
    
    // 创建测试目录和文件
    let test_dir = "test_ui_progress";
    if std::path::Path::new(test_dir).exists() {
        fs::remove_dir_all(test_dir)?;
    }
    fs::create_dir_all(test_dir)?;
    
    // 创建一些测试文件
    create_test_files(test_dir)?;
    
    // 测试异步进度更新
    test_async_progress_updates(test_dir)?;
    
    println!("\n🎉 UI进度测试完成！");
    
    Ok(())
}

fn create_test_files(test_dir: &str) -> Result<(), Box<dyn std::error::Error>> {
    println!("\n📁 创建测试文件...");
    
    // 创建不同大小的文件来测试进度显示
    let test_files = vec![
        ("file1.txt", "小文件内容。".repeat(1000)),      // ~12KB
        ("file2.txt", "中等文件内容。".repeat(10000)),   // ~120KB  
        ("file3.txt", "大文件内容。".repeat(50000)),     // ~600KB
    ];
    
    for (filename, content) in test_files {
        let file_path = format!("{}/{}", test_dir, filename);
        fs::write(&file_path, content)?;
        
        let size = fs::metadata(&file_path)?.len();
        println!("  ✅ 创建文件: {} ({:.2} KB)", filename, size as f64 / 1024.0);
    }
    
    Ok(())
}

fn test_async_progress_updates(test_dir: &str) -> Result<(), Box<dyn std::error::Error>> {
    println!("\n🔄 测试异步进度更新...");
    
    // 创建设置
    let settings = Settings {
        password: "ui_test_password".to_string(),
        operation_mode: OperationMode::Encrypt,
        encryption_algorithm: EncryptionAlgorithm::AES256,
        max_threads: 1, // 使用单线程以便更好地观察进度
        encrypt_filename: false,
        delete_source: false,
        file_extension: "uitest".to_string(),
    };
    
    // 加载文件
    let files = FileManager::load_files_from_directory(test_dir);
    let mut selected_files = files.clone();
    
    // 选择所有文件进行加密
    for file in &mut selected_files {
        file.selected = true;
    }
    
    println!("  📊 开始异步加密操作...");
    
    // 启动异步加密
    match CryptoEngine::start_operation_async(settings, selected_files, None) {
        Ok(mut handle) => {
            println!("  🚀 异步操作已启动");
            
            // 模拟UI更新循环
            let mut update_count = 0;
            loop {
                // 检查进度更新（模拟UI的check_operation_status）
                let mut has_progress_update = false;
                while let Some(progress_info) = handle.try_recv_progress() {
                    has_progress_update = true;
                    print_progress_update(&progress_info, update_count);
                    update_count += 1;
                }
                
                // 检查操作是否完成
                if handle.is_finished() {
                    let status = handle.status();
                    println!("  🏁 操作完成，状态: {:?}", status);
                    break;
                }
                
                // 模拟UI刷新间隔
                thread::sleep(Duration::from_millis(50));
            }
            
            println!("  ✅ 总共收到 {} 次进度更新", update_count);
        }
        Err(e) => {
            println!("  ❌ 启动异步操作失败: {}", e);
        }
    }
    
    Ok(())
}

fn print_progress_update(progress: &ProgressInfo, update_count: usize) {
    println!("  📈 进度更新 #{}: ", update_count + 1);
    println!("    - 当前文件: {} ({}/{})", 
        progress.current_file,
        progress.current_file_index + 1,
        progress.total_files
    );
    println!("    - 文件进度: {:.1}% | 总体进度: {:.1}%", 
        progress.current_file_progress * 100.0,
        progress.overall_progress * 100.0
    );
    println!("    - 处理速度: {:.2} MB/s", progress.speed_mbps);
    println!("    - 已用时间: {:.1}s", progress.elapsed_time);
    if progress.estimated_remaining > 0.0 {
        println!("    - 预计剩余: {:.1}s", progress.estimated_remaining);
    }
    println!("    - 数据量: {:.2} MB / {:.2} MB", 
        progress.processed_bytes as f64 / (1024.0 * 1024.0),
        progress.total_bytes as f64 / (1024.0 * 1024.0)
    );
    println!("    ---");
}
