use std::path::PathBuf;
use std::sync::{Arc, atomic::AtomicBool, mpsc};
use std::thread::JoinHandle;

#[derive(Debug, Clone, PartialEq)]
pub enum OperationMode {
    Encrypt,
    Decrypt,
}

#[derive(Debug, Clone, PartialEq)]
pub enum EncryptionAlgorithm {
    AES256,
    ChaCha20,
}

#[derive(Debug, Clone)]
pub struct FileItem {
    pub path: PathBuf,
    pub selected: bool,
    pub name: String,
}

#[derive(Debug, Clone, PartialEq)]
pub enum AppState {
    Idle,
    Running,
    Paused,
}

/// 异步操作状态
#[derive(Debug, Clone, PartialEq)]
pub enum OperationStatus {
    Running,
    Completed,
    Failed(String),
    Cancelled,
}

/// 进度信息
#[derive(Debug, Clone)]
pub struct ProgressInfo {
    pub current_file: String,
    pub current_file_index: usize,
    pub total_files: usize,
    pub current_file_progress: f32,  // 0.0 - 1.0
    pub overall_progress: f32,       // 0.0 - 1.0
    pub current_file_size: u64,      // 当前文件大小（字节）
    pub processed_bytes: u64,        // 已处理字节数
    pub total_bytes: u64,            // 总字节数
    pub speed_mbps: f64,             // 处理速度（MB/s）
    pub elapsed_time: f64,           // 已用时间（秒）
    pub estimated_remaining: f64,    // 预计剩余时间（秒）
}

/// 进度回调函数类型
pub type ProgressCallback = Arc<dyn Fn(ProgressInfo) + Send + Sync>;

/// 异步操作句柄
pub struct OperationHandle {
    pub(crate) thread_handle: Option<JoinHandle<Result<(), String>>>,
    pub(crate) should_stop: Arc<AtomicBool>,
    pub(crate) should_skip: Arc<AtomicBool>,
    pub(crate) status: Arc<std::sync::Mutex<OperationStatus>>,
    pub(crate) progress: Arc<std::sync::Mutex<ProgressInfo>>,
    pub(crate) progress_receiver: Option<mpsc::Receiver<ProgressInfo>>,
}

impl OperationHandle {
    /// 获取当前操作状态
    pub fn status(&self) -> OperationStatus {
        self.status.lock().unwrap().clone()
    }

    /// 获取当前进度信息
    pub fn progress(&self) -> ProgressInfo {
        self.progress.lock().unwrap().clone()
    }

    /// 请求停止操作
    pub fn stop(&self) {
        self.should_stop.store(true, std::sync::atomic::Ordering::Relaxed);
    }

    /// 请求跳过当前文件
    pub fn skip_current(&self) {
        self.should_skip.store(true, std::sync::atomic::Ordering::Relaxed);
    }

    /// 等待操作完成
    pub fn wait(mut self) -> Result<(), String> {
        if let Some(handle) = self.thread_handle.take() {
            handle.join().map_err(|_| "Thread panicked".to_string())?
        } else {
            Ok(())
        }
    }

    /// 检查操作是否完成
    pub fn is_finished(&self) -> bool {
        matches!(self.status(), OperationStatus::Completed | OperationStatus::Failed(_) | OperationStatus::Cancelled)
    }

    /// 尝试接收进度更新（非阻塞）
    pub fn try_recv_progress(&mut self) -> Option<ProgressInfo> {
        if let Some(ref receiver) = self.progress_receiver {
            receiver.try_recv().ok()
        } else {
            None
        }
    }
}

/// 应用设置结构体
#[derive(Debug, Clone)]
pub struct Settings {
    pub operation_mode: OperationMode,
    pub encryption_algorithm: EncryptionAlgorithm,
    pub password: String,
    pub max_threads: u32,
    pub encrypt_filename: bool,
    pub delete_source: bool,
    pub file_extension: String,
}

/// 文件管理结构体
#[derive(Debug, Clone)]
#[derive(Default)]
pub struct FileManagerState {
    pub left_directory: String,
    pub right_directory: String,
    pub left_files: Vec<FileItem>,
    pub right_files: Vec<FileItem>,
}

/// 进度状态结构体
#[derive(Debug, Clone)]
pub struct ProgressState {
    pub current_progress: f32,
    pub total_progress: f32,
    pub current_file_name: String,
    pub current_file_index: usize,
    pub total_files: usize,
    pub current_file_size: u64,
    pub processed_bytes: u64,
    pub total_bytes: u64,
    pub speed_mbps: f64,
    pub elapsed_time: f64,
    pub estimated_remaining: f64,
}

/// 对话框状态结构体
#[derive(Debug, Clone)]
#[derive(Default)]
pub struct DialogState {
    pub show_error_dialog: bool,
    pub show_complete_dialog: bool,
    pub error_message: String,
}

impl Default for Settings {
    fn default() -> Self {
        Self {
            operation_mode: OperationMode::Encrypt,
            encryption_algorithm: EncryptionAlgorithm::AES256,
            password: String::new(),
            max_threads: 1,
            encrypt_filename: true,
            delete_source: true,
            file_extension: "enc".to_string(),
        }
    }
}


impl Default for ProgressState {
    fn default() -> Self {
        Self {
            current_progress: 0.0,
            total_progress: 0.0,
            current_file_name: String::new(),
            current_file_index: 0,
            total_files: 0,
            current_file_size: 0,
            processed_bytes: 0,
            total_bytes: 0,
            speed_mbps: 0.0,
            elapsed_time: 0.0,
            estimated_remaining: 0.0,
        }
    }
}


impl FileItem {
    pub fn new(path: PathBuf, name: String) -> Self {
        Self {
            path,
            selected: false,
            name,
        }
    }
}

impl std::fmt::Display for EncryptionAlgorithm {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            EncryptionAlgorithm::AES256 => write!(f, "AES-256"),
            EncryptionAlgorithm::ChaCha20 => write!(f, "ChaCha20"),
        }
    }
} 