use crate::models::{FileItem, Settings, OperationMode, OperationHandle, OperationStatus, ProgressInfo, ProgressCallback};
use crate::progress::{ProgressManager, ProgressTracker};
use super::traits::{CryptoProvider, CryptoResult, CryptoError};
use super::create_crypto_provider;
use std::fs::File;
use std::io::{BufReader, BufWriter};
use std::path::PathBuf;
use std::fs;
use std::sync::{Arc, atomic::AtomicBool, Mutex, mpsc};
use std::thread;

use rand::RngCore;
use aes_gcm::aead::OsRng;
use threadpool::ThreadPool;

/// 重构后的加密引擎，使用策略模式和线程池
pub struct CryptoEngine {
    thread_pool: Arc<ThreadPool>,
}

impl CryptoEngine {
    /// 创建新的加密引擎实例
    pub fn new(max_threads: usize) -> Self {
        let thread_pool = ThreadPool::new(max_threads);
        Self {
            thread_pool: Arc::new(thread_pool),
        }
    }

    /// 从设置创建加密引擎实例
    pub fn from_settings(settings: &Settings) -> Self {
        Self::new(settings.max_threads as usize)
    }

    /// 开始异步加密/解密操作
    pub fn start_operation_async(
        &self,
        settings: Settings,
        files: Vec<FileItem>,
        progress_callback: Option<ProgressCallback>,
    ) -> Result<OperationHandle, String> {
        // 验证密码不为空
        if settings.password.is_empty() {
            return Err("Password cannot be empty".to_string());
        }

        // 筛选已选中的文件
        let selected_files: Vec<FileItem> = files.into_iter()
            .filter(|file| file.selected)
            .collect();

        if selected_files.is_empty() {
            return Err("No files selected".to_string());
        }

        // 创建控制标志
        let should_stop = Arc::new(AtomicBool::new(false));
        let should_skip = Arc::new(AtomicBool::new(false));
        let status = Arc::new(Mutex::new(OperationStatus::Running));
        let progress = Arc::new(Mutex::new(ProgressInfo {
            current_file: String::new(),
            current_file_index: 0,
            total_files: selected_files.len(),
            current_file_progress: 0.0,
            overall_progress: 0.0,
            current_file_size: 0,
            processed_bytes: 0,
            total_bytes: 0,
            speed_mbps: 0.0,
            elapsed_time: 0.0,
            estimated_remaining: 0.0,
        }));

        // 创建进度消息通道
        let (progress_sender, progress_receiver) = mpsc::channel::<ProgressInfo>();

        // 创建进度跟踪器
        let progress_tracker = ProgressManager::create_tracker(
            &selected_files,
            progress_sender,
            progress_callback,
            Some(progress.clone()),
        );

        // 克隆用于线程的引用
        let should_stop_clone = should_stop.clone();
        let should_skip_clone = should_skip.clone();
        let status_clone = status.clone();
        let thread_pool_clone = self.thread_pool.clone();

        // 启动工作线程
        let thread_handle = thread::spawn(move || {
            Self::process_files_async_with_pool(
                &settings,
                &selected_files,
                should_stop_clone,
                should_skip_clone,
                status_clone,
                progress_tracker,
                thread_pool_clone,
            )
        });

        Ok(OperationHandle {
            thread_handle: Some(thread_handle),
            should_stop,
            should_skip,
            status,
            progress,
            progress_receiver: Some(progress_receiver),
        })
    }

    /// 同步版本的开始加密/解密操作（实例方法）
    pub fn start_operation(
        &self,
        settings: &Settings,
        files: &[FileItem],
    ) -> Result<(), String> {
        // 验证密码不为空
        if settings.password.is_empty() {
            return Err("Password cannot be empty".to_string());
        }

        // 筛选已选中的文件
        let selected_files: Vec<&FileItem> = files.iter()
            .filter(|file| file.selected)
            .collect();

        if selected_files.is_empty() {
            return Err("No files selected".to_string());
        }

        // 根据是否启用多线程决定处理方式
        if settings.max_threads > 1 {
            self.process_files_with_pool(settings, &selected_files)
        } else {
            Self::process_files_sequential(settings, &selected_files)
        }
    }
    
    /// 顺序处理文件
    fn process_files_sequential(settings: &Settings, files: &[&FileItem]) -> Result<(), String> {
        for file in files {
            match settings.operation_mode {
                OperationMode::Encrypt => {
                    Self::encrypt_file(settings, file)?;
                }
                OperationMode::Decrypt => {
                    Self::decrypt_file(settings, file)?;
                }
            }
        }
        Ok(())
    }


    /// 使用线程池处理文件
    fn process_files_with_pool(&self, settings: &Settings, files: &[&FileItem]) -> Result<(), String> {
        use std::sync::mpsc;

        let (tx, rx) = mpsc::channel();

        // 为每个文件提交任务到线程池
        for file in files {
            let tx = tx.clone();
            let settings = settings.clone();
            let file = (*file).clone();

            self.thread_pool.execute(move || {
                let result = match settings.operation_mode {
                    OperationMode::Encrypt => {
                        Self::encrypt_file(&settings, &file)
                    }
                    OperationMode::Decrypt => {
                        Self::decrypt_file(&settings, &file)
                    }
                };
                tx.send(result).unwrap();
            });
        }

        // 等待所有任务完成并收集结果
        drop(tx); // 关闭发送端
        for _ in 0..files.len() {
            match rx.recv() {
                Ok(result) => result?,
                Err(_) => return Err("Failed to receive result from thread pool".to_string()),
            }
        }

        Ok(())
    }

    /// 异步处理文件（带进度回调和取消支持，使用线程池）
    fn process_files_async_with_pool(
        settings: &Settings,
        files: &[FileItem],
        should_stop: Arc<AtomicBool>,
        should_skip: Arc<AtomicBool>,
        status: Arc<Mutex<OperationStatus>>,
        progress_tracker: ProgressTracker,
        thread_pool: Arc<ThreadPool>,
    ) -> Result<(), String> {
        use std::sync::mpsc;

        let (tx, rx) = mpsc::channel();
        let mut pending_tasks = 0;

        for (index, file) in files.iter().enumerate() {
            // 检查是否应该停止
            if should_stop.load(std::sync::atomic::Ordering::Relaxed) {
                *status.lock().unwrap() = OperationStatus::Cancelled;
                return Err("Operation cancelled".to_string());
            }

            // 获取当前文件大小
            let current_file_size = fs::metadata(&file.path)
                .map(|m| m.len())
                .unwrap_or(0);

            // 开始处理文件
            progress_tracker.start_file(index, file.name.clone(), current_file_size);

            // 提交任务到线程池
            let tx = tx.clone();
            let settings = settings.clone();
            let file = file.clone();
            let should_stop_clone = should_stop.clone();
            let should_skip_clone = should_skip.clone();

            thread_pool.execute(move || {
                // 在任务执行前再次检查是否应该停止
                if should_stop_clone.load(std::sync::atomic::Ordering::Relaxed) {
                    tx.send((index, Err("Operation cancelled".to_string()))).unwrap();
                    return;
                }

                // 检查是否跳过当前文件
                if should_skip_clone.load(std::sync::atomic::Ordering::Relaxed) {
                    should_skip_clone.store(false, std::sync::atomic::Ordering::Relaxed);
                    tx.send((index, Ok(()))).unwrap();
                    return;
                }

                // 处理单个文件
                let result = match settings.operation_mode {
                    OperationMode::Encrypt => {
                        Self::encrypt_file(&settings, &file)
                    }
                    OperationMode::Decrypt => {
                        Self::decrypt_file(&settings, &file)
                    }
                };

                tx.send((index, result)).unwrap();
            });

            pending_tasks += 1;
        }

        // 等待所有任务完成
        drop(tx); // 关闭发送端
        for _ in 0..pending_tasks {
            match rx.recv() {
                Ok((index, result)) => {
                    // 检查是否应该停止
                    if should_stop.load(std::sync::atomic::Ordering::Relaxed) {
                        *status.lock().unwrap() = OperationStatus::Cancelled;
                        return Err("Operation cancelled".to_string());
                    }

                    // 处理结果
                    if let Err(e) = result {
                        *status.lock().unwrap() = OperationStatus::Failed(e.clone());
                        return Err(e);
                    }

                    // 完成文件处理
                    if let Some(file) = files.get(index) {
                        let file_size = fs::metadata(&file.path)
                            .map(|m| m.len())
                            .unwrap_or(0);
                        progress_tracker.complete_file(file_size);
                    }
                }
                Err(_) => {
                    *status.lock().unwrap() = OperationStatus::Failed("Failed to receive result from thread pool".to_string());
                    return Err("Failed to receive result from thread pool".to_string());
                }
            }
        }

        // 操作完成
        *status.lock().unwrap() = OperationStatus::Completed;
        Ok(())
    }


    
    /// 加密单个文件
    fn encrypt_file(settings: &Settings, file: &FileItem) -> Result<(), String> {
        let input_path = &file.path;
        let output_path = Self::generate_output_path(settings, file, true)?;
        
        // 打开输入文件
        let input_file = File::open(input_path)
            .map_err(|e| format!("Failed to open file '{}': {}", file.name, e))?;
        let mut reader = BufReader::new(input_file);

        // 创建输出文件
        let output_file = File::create(&output_path)
            .map_err(|e| format!("Failed to create output file: {}", e))?;
        let mut writer = BufWriter::new(output_file);

        // 使用策略模式进行加密
        let crypto_provider = create_crypto_provider(&settings.encryption_algorithm);
        crypto_provider.encrypt_stream(&settings.password, &mut reader, &mut writer)
            .map_err(|e| format!("Failed to encrypt file '{}': {}", file.name, e))?;

        // 如果设置删除源文件
        if settings.delete_source {
            fs::remove_file(input_path)
                .map_err(|e| format!("Failed to delete source file: {}", e))?;
        }
        
        Ok(())
    }
    
    /// 解密单个文件
    fn decrypt_file(settings: &Settings, file: &FileItem) -> Result<(), String> {
        let input_path = &file.path;
        let output_path = Self::generate_output_path(settings, file, false)?;

        // 打开输入文件
        let input_file = File::open(input_path)
            .map_err(|e| format!("Failed to open file '{}': {}", file.name, e))?;
        let mut reader = BufReader::new(input_file);

        // 创建输出文件
        let output_file = File::create(&output_path)
            .map_err(|e| format!("Failed to create output file: {}", e))?;
        let mut writer = BufWriter::new(output_file);

        // 使用策略模式进行解密
        let crypto_provider = create_crypto_provider(&settings.encryption_algorithm);
        crypto_provider.decrypt_stream(&settings.password, &mut reader, &mut writer)
            .map_err(|e| format!("Failed to decrypt file '{}': {}", file.name, e))?;

        // 如果设置删除源文件
        if settings.delete_source {
            fs::remove_file(input_path)
                .map_err(|e| format!("Failed to delete source file: {}", e))?;
        }

        Ok(())
    }


    
    /// 生成输出文件路径
    fn generate_output_path(settings: &Settings, file: &FileItem, is_encrypt: bool) -> Result<PathBuf, String> {
        let input_path = &file.path;
        let mut output_path = input_path.clone();
        
        if is_encrypt {
            // 加密：生成输出文件名
            if settings.encrypt_filename {
                // 如果加密文件名，生成随机文件名
                let mut random_name = [0u8; 16];
                OsRng.fill_bytes(&mut random_name);
                let random_name = hex::encode(random_name);
                output_path.set_file_name(format!("{}.{}", random_name, settings.file_extension));
            } else {
                // 否则只添加扩展名
                let original_name = file.name.clone();
                output_path.set_file_name(format!("{}.{}", original_name, settings.file_extension));
            }
        } else {
            // 解密：生成输出文件名（移除加密扩展名）
            let file_name = file.name.clone();
            if file_name.ends_with(&format!(".{}", settings.file_extension)) {
                let original_name = file_name.trim_end_matches(&format!(".{}", settings.file_extension));
                output_path.set_file_name(original_name);
            } else {
                output_path.set_file_name(format!("{}.decrypted", file_name));
            }
        }
        
        Ok(output_path)
    }
    
    /// 获取加密算法信息
    pub fn get_algorithm_info(settings: &Settings) -> String {
        let provider = create_crypto_provider(&settings.encryption_algorithm);
        format!("Algorithm: {}, Chunk size: {} MB",
                provider.algorithm_name(),
                provider.chunk_size() / (1024 * 1024))
    }
    
    /// 验证密码是否正确（通过尝试读取加密文件头）
    pub fn verify_password(settings: &Settings, file_path: &std::path::Path) -> CryptoResult<bool> {
        let provider = create_crypto_provider(&settings.encryption_algorithm);
        let file_data = fs::read(file_path)
            .map_err(CryptoError::IoError)?;
        
        if file_data.len() < 32 {
            return Ok(false); // File too small to be a valid encrypted file
        }
        
        provider.verify_password(&settings.password, &file_data[0..64])
    }
    
    /// 静态方法：同步版本的开始加密/解密操作（保持向后兼容）
    pub fn start_operation_static(
        settings: &Settings,
        files: &[FileItem],
    ) -> Result<(), String> {
        let engine = Self::from_settings(settings);
        engine.start_operation(settings, files)
    }

    /// 静态方法：开始异步加密/解密操作（保持向后兼容）
    pub fn start_operation_async_static(
        settings: Settings,
        files: Vec<FileItem>,
        progress_callback: Option<ProgressCallback>,
    ) -> Result<OperationHandle, String> {
        let engine = Self::from_settings(&settings);
        engine.start_operation_async(settings, files, progress_callback)
    }


} 