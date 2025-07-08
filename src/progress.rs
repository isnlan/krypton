use std::sync::{Arc, Mutex, mpsc};
use std::time::Instant;
use crate::models::{ProgressInfo, ProgressCallback};

/// 进度跟踪器 - 负责管理和计算进度信息
pub struct ProgressTracker {
    /// 进度信息的共享状态
    progress_state: Arc<Mutex<ProgressInfo>>,
    /// 进度消息发送器
    progress_sender: mpsc::Sender<ProgressInfo>,
    /// 操作开始时间
    start_time: Instant,
    /// 进度回调函数
    callback: Option<ProgressCallback>,
    /// 外部共享的进度状态（用于与UI同步）
    external_progress: Option<Arc<Mutex<ProgressInfo>>>,
}

impl ProgressTracker {
    /// 创建新的进度跟踪器
    pub fn new(
        total_files: usize,
        total_bytes: u64,
        progress_sender: mpsc::Sender<ProgressInfo>,
        callback: Option<ProgressCallback>,
        external_progress: Option<Arc<Mutex<ProgressInfo>>>,
    ) -> Self {
        let progress_state = Arc::new(Mutex::new(ProgressInfo {
            current_file: String::new(),
            current_file_index: 0,
            total_files,
            current_file_progress: 0.0,
            overall_progress: 0.0,
            current_file_size: 0,
            processed_bytes: 0,
            total_bytes,
            speed_mbps: 0.0,
            elapsed_time: 0.0,
            estimated_remaining: 0.0,
        }));

        Self {
            progress_state,
            progress_sender,
            start_time: Instant::now(),
            callback,
            external_progress,
        }
    }

    /// 开始处理新文件
    pub fn start_file(&self, file_index: usize, file_name: String, file_size: u64) {
        let mut progress = self.progress_state.lock().unwrap();
        progress.current_file = file_name;
        progress.current_file_index = file_index;
        progress.current_file_size = file_size;
        progress.current_file_progress = 0.0;
        progress.overall_progress = file_index as f32 / progress.total_files as f32;
        
        self.update_timing(&mut progress);
        drop(progress);
        
        self.send_update();
    }

    /// 完成当前文件处理
    pub fn complete_file(&self, file_size: u64) {
        let mut progress = self.progress_state.lock().unwrap();
        progress.current_file_progress = 1.0;
        progress.processed_bytes += file_size;
        progress.overall_progress = (progress.current_file_index + 1) as f32 / progress.total_files as f32;
        
        self.update_timing(&mut progress);
        drop(progress);
        
        self.send_update();
    }

    /// 更新文件内部进度（0.0 - 1.0）
    pub fn update_file_progress(&self, progress_ratio: f32) {
        let mut progress = self.progress_state.lock().unwrap();
        progress.current_file_progress = progress_ratio.clamp(0.0, 1.0);
        
        self.update_timing(&mut progress);
        drop(progress);
        
        self.send_update();
    }

    /// 获取当前进度信息的副本
    pub fn get_progress(&self) -> ProgressInfo {
        self.progress_state.lock().unwrap().clone()
    }

    /// 更新时间相关的计算
    fn update_timing(&self, progress: &mut ProgressInfo) {
        let elapsed = self.start_time.elapsed().as_secs_f64();
        progress.elapsed_time = elapsed;

        if elapsed > 0.0 && progress.processed_bytes > 0 {
            // 计算处理速度（MB/s）
            progress.speed_mbps = (progress.processed_bytes as f64) / (1024.0 * 1024.0) / elapsed;

            // 估算剩余时间
            let remaining_bytes = progress.total_bytes - progress.processed_bytes;
            if progress.speed_mbps > 0.0 {
                progress.estimated_remaining = (remaining_bytes as f64) / (1024.0 * 1024.0) / progress.speed_mbps;
            } else {
                progress.estimated_remaining = 0.0;
            }
        }
    }

    /// 发送进度更新
    fn send_update(&self) {
        let progress_info = self.progress_state.lock().unwrap().clone();

        // 同步到外部进度状态
        if let Some(ref external_progress) = self.external_progress {
            if let Ok(mut external) = external_progress.lock() {
                *external = progress_info.clone();
            }
        }

        // 调用回调函数
        if let Some(ref callback) = self.callback {
            callback(progress_info.clone());
        }

        // 发送到UI线程
        let _ = self.progress_sender.send(progress_info);
    }
}

/// 进度管理器 - 负责创建和管理进度跟踪器
pub struct ProgressManager;

impl ProgressManager {
    /// 计算文件列表的总大小
    pub fn calculate_total_size(files: &[crate::models::FileItem]) -> u64 {
        files.iter()
            .map(|file| {
                std::fs::metadata(&file.path)
                    .map(|m| m.len())
                    .unwrap_or(0)
            })
            .sum()
    }

    /// 创建进度跟踪器
    pub fn create_tracker(
        files: &[crate::models::FileItem],
        progress_sender: mpsc::Sender<ProgressInfo>,
        callback: Option<ProgressCallback>,
        external_progress: Option<Arc<Mutex<ProgressInfo>>>,
    ) -> ProgressTracker {
        let total_files = files.len();
        let total_bytes = Self::calculate_total_size(files);

        ProgressTracker::new(total_files, total_bytes, progress_sender, callback, external_progress)
    }
}

/// 进度格式化工具
pub struct ProgressFormatter;

impl ProgressFormatter {
    /// 格式化时间显示（秒转换为时:分:秒）
    pub fn format_time(seconds: f64) -> String {
        if seconds < 60.0 {
            format!("{:.0}s", seconds)
        } else if seconds < 3600.0 {
            let minutes = (seconds / 60.0) as u32;
            let secs = (seconds % 60.0) as u32;
            format!("{}m {}s", minutes, secs)
        } else {
            let hours = (seconds / 3600.0) as u32;
            let minutes = ((seconds % 3600.0) / 60.0) as u32;
            let secs = (seconds % 60.0) as u32;
            format!("{}h {}m {}s", hours, minutes, secs)
        }
    }

    /// 格式化字节大小显示
    pub fn format_bytes(bytes: u64) -> String {
        const UNITS: &[&str] = &["B", "KB", "MB", "GB", "TB"];
        let mut size = bytes as f64;
        let mut unit_index = 0;

        while size >= 1024.0 && unit_index < UNITS.len() - 1 {
            size /= 1024.0;
            unit_index += 1;
        }

        if unit_index == 0 {
            format!("{} {}", bytes, UNITS[unit_index])
        } else {
            format!("{:.2} {}", size, UNITS[unit_index])
        }
    }

    /// 格式化速度显示
    pub fn format_speed(mbps: f64) -> String {
        if mbps < 0.01 {
            "0.00 MB/s".to_string()
        } else if mbps < 1.0 {
            format!("{:.2} MB/s", mbps)
        } else if mbps < 1024.0 {
            format!("{:.1} MB/s", mbps)
        } else {
            format!("{:.2} GB/s", mbps / 1024.0)
        }
    }

    /// 格式化百分比显示
    pub fn format_percentage(ratio: f32) -> String {
        format!("{:.1}%", (ratio * 100.0).clamp(0.0, 100.0))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_format_time() {
        assert_eq!(ProgressFormatter::format_time(30.0), "30s");
        assert_eq!(ProgressFormatter::format_time(90.0), "1m 30s");
        assert_eq!(ProgressFormatter::format_time(3661.0), "1h 1m 1s");
    }

    #[test]
    fn test_format_bytes() {
        assert_eq!(ProgressFormatter::format_bytes(512), "512 B");
        assert_eq!(ProgressFormatter::format_bytes(1536), "1.50 KB");
        assert_eq!(ProgressFormatter::format_bytes(1048576), "1.00 MB");
    }

    #[test]
    fn test_format_speed() {
        assert_eq!(ProgressFormatter::format_speed(0.005), "0.00 MB/s");
        assert_eq!(ProgressFormatter::format_speed(0.5), "0.50 MB/s");
        assert_eq!(ProgressFormatter::format_speed(10.5), "10.5 MB/s");
        assert_eq!(ProgressFormatter::format_speed(1536.0), "1.50 GB/s");
    }

    #[test]
    fn test_format_percentage() {
        assert_eq!(ProgressFormatter::format_percentage(0.0), "0.0%");
        assert_eq!(ProgressFormatter::format_percentage(0.5), "50.0%");
        assert_eq!(ProgressFormatter::format_percentage(1.0), "100.0%");
    }
}
