use std::path::PathBuf;

#[derive(Debug, Clone, PartialEq)]
pub enum OperationMode {
    Encrypt,
    Decrypt,
}

#[derive(Debug, Clone, PartialEq)]
pub enum EncryptionAlgorithm {
    AES256,
    ChaCha20,
    Blowfish,
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
}

/// 对话框状态结构体
#[derive(Debug, Clone)]
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

impl Default for FileManagerState {
    fn default() -> Self {
        Self {
            left_directory: String::new(),
            right_directory: String::new(),
            left_files: Vec::new(),
            right_files: Vec::new(),
        }
    }
}

impl Default for ProgressState {
    fn default() -> Self {
        Self {
            current_progress: 0.0,
            total_progress: 0.0,
            current_file_name: String::new(),
        }
    }
}

impl Default for DialogState {
    fn default() -> Self {
        Self {
            show_error_dialog: false,
            show_complete_dialog: false,
            error_message: String::new(),
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
            EncryptionAlgorithm::Blowfish => write!(f, "Blowfish"),
        }
    }
} 