use crate::models::{FileItem, Settings, OperationMode};
use super::traits::{CryptoProvider, CryptoResult, CryptoError};
use super::{create_crypto_provider, CryptoProviderEnum};
use std::fs::File;
use std::io::{BufReader, BufWriter};
use std::path::PathBuf;
use std::fs;
use rayon::prelude::*;
use rand::RngCore;
use aes_gcm::aead::OsRng;

/// 重构后的加密引擎，使用策略模式
pub struct CryptoEngine;

impl CryptoEngine {
    /// 开始加密/解密操作
    pub fn start_operation(
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
            Self::process_files_parallel(settings, &selected_files)
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
    
    /// 并行处理文件
    fn process_files_parallel(settings: &Settings, files: &[&FileItem]) -> Result<(), String> {
        let results: Vec<Result<(), String>> = files.par_iter()
            .map(|file| {
                match settings.operation_mode {
                    OperationMode::Encrypt => {
                        Self::encrypt_file(settings, file)
                    }
                    OperationMode::Decrypt => {
                        Self::decrypt_file(settings, file)
                    }
                }
            })
            .collect();
            
        // 检查是否有错误
        for result in results {
            result?;
        }
        
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
            .map_err(|e| CryptoError::IoError(e))?;
        
        if file_data.len() < 32 {
            return Ok(false); // File too small to be a valid encrypted file
        }
        
        provider.verify_password(&settings.password, &file_data[0..64])
    }
    
    pub fn stop_operation() {
        // 停止操作的逻辑
        // 可以使用原子标志来控制加密过程的停止
    }
    
    pub fn skip_current_task() {
        // 跳过当前任务的逻辑
        // 可以使用原子标志来控制单个文件处理的跳过
    }
} 