use crate::models::{FileItem, Settings};
use std::path::PathBuf;
use std::fs;

pub struct FileManager;

impl FileManager {
    pub fn load_files_from_directory(directory: &str) -> Vec<FileItem> {
        // 检查目录路径是否为空
        if directory.is_empty() {
            return Vec::new();
        }

        // 规范化路径
        let path = std::path::Path::new(directory);
        if !path.exists() {
            eprintln!("目录 '{}' 不存在", directory);
            return Vec::new();
        }
        
        if !path.is_dir() {
            eprintln!("路径 '{}' 不是一个目录", directory);
            return Vec::new();
        }
        
        // 尝试读取目录内容
        match fs::read_dir(directory) {
            Ok(entries) => {
                let mut files = Vec::new();
                
                for entry in entries {
                    match entry {
                        Ok(entry) => {
                            let path = entry.path();
                            
                            // 只处理文件，跳过目录
                            if path.is_file() {
                                if let Some(file_name) = path.file_name() {
                                    if let Some(name_str) = file_name.to_str() {
                                        files.push(FileItem::new(
                                            path.clone(),
                                            name_str.to_string(),
                                        ));
                                    }
                                }
                            }
                        }
                        Err(e) => {
                            eprintln!("读取目录项时出错: {}", e);
                        }
                    }
                }
                
                // 按文件名排序
                files.sort_by(|a, b| a.name.cmp(&b.name));
                files
            }
            Err(e) => {
                eprintln!("读取目录 '{}' 时出错: {}", directory, e);
                Vec::new()
            }
        }
    }
    
    pub fn load_encrypted_files_from_directory(directory: &str, settings: &Settings) -> Vec<FileItem> {
        // 检查目录路径是否为空
        if directory.is_empty() {
            return Vec::new();
        }
        
        // 规范化路径
        let path = std::path::Path::new(directory);
        if !path.exists() {
            eprintln!("目录 '{}' 不存在", directory);
            return Vec::new();
        }
        
        if !path.is_dir() {
            eprintln!("路径 '{}' 不是一个目录", directory);
            return Vec::new();
        }
        
        // 尝试读取目录内容，筛选加密文件
        match fs::read_dir(directory) {
            Ok(entries) => {
                let mut files = Vec::new();
                
                for entry in entries {
                    match entry {
                        Ok(entry) => {
                            let path = entry.path();
                            
                            // 只处理文件，跳过目录
                            if path.is_file() {
                                if let Some(file_name) = path.file_name() {
                                    if let Some(name_str) = file_name.to_str() {
                                        // 筛选加密文件（以指定后缀结尾）
                                        let extension_with_dot = format!(".{}", settings.file_extension);
                                        if name_str.ends_with(&extension_with_dot) {
                                            files.push(FileItem::new(
                                                path.clone(),
                                                name_str.to_string(),
                                            ));
                                        }
                                    }
                                }
                            }
                        }
                        Err(e) => {
                            eprintln!("读取目录项时出错: {}", e);
                        }
                    }
                }
                
                // 按文件名排序
                files.sort_by(|a, b| a.name.cmp(&b.name));
                files
            }
            Err(e) => {
                eprintln!("读取目录 '{}' 时出错: {}", directory, e);
                Vec::new()
            }
        }
    }
}

pub struct CryptoEngine;

impl CryptoEngine {
    pub fn start_operation(
        _settings: &Settings,
        _files: &[FileItem],
    ) -> Result<(), String> {
        // 实际的加密/解密逻辑应该在这里实现
        // 目前返回演示错误
        Err("Demo error: Unable to access file".to_string())
    }
    
    pub fn stop_operation() {
        // 停止操作的逻辑
    }
    
    pub fn skip_current_task() {
        // 跳过当前任务的逻辑
    }
} 