use eframe::egui;
use crate::models::{OperationMode, FileItem, AppState, Settings, FileManagerState, ProgressState, DialogState, OperationHandle, ProgressInfo, ProgressCallback};
use crate::core::FileManager;
use crate::crypto::CryptoEngine;
use crate::ui::{SettingsPanel, FilePanel, ProgressPanel, ControlPanel, ErrorDialog, CompleteDialog, PanelEvent, DialogEvent};
use rfd::FileDialog;
use std::sync::Arc;

pub struct KryptonApp {
    // 应用设置
    settings: Settings,

    // 文件管理状态
    file_manager: FileManagerState,

    // 进度状态
    progress: ProgressState,

    // 应用状态
    app_state: AppState,

    // 对话框状态
    dialog: DialogState,

    // 异步操作句柄
    operation_handle: Option<OperationHandle>,
}

impl Default for KryptonApp {
    fn default() -> Self {
        Self {
            settings: Settings::default(),
            file_manager: FileManagerState::default(),
            progress: ProgressState::default(),
            app_state: AppState::Idle,
            dialog: DialogState::default(),
            operation_handle: None,
        }
    }
}

impl KryptonApp {
    pub fn new() -> Self {
        Self::default()
    }
    
    fn load_left_files(&mut self) {
        self.file_manager.left_files = FileManager::load_files_from_directory(&self.file_manager.left_directory);
    }
    
    fn load_right_files(&mut self) {
        self.file_manager.right_files = FileManager::load_encrypted_files_from_directory(&self.file_manager.right_directory, &self.settings);
    }
    
    fn start_operation(&mut self) {
        self.app_state = AppState::Running;
        self.progress.current_progress = 0.0;
        self.progress.total_progress = 0.0;
        self.progress.current_file_name = "Starting processing...".to_string();

        // Get selected files based on operation mode
        let selected_files: Vec<FileItem> = match self.settings.operation_mode {
            OperationMode::Encrypt => self.file_manager.left_files.iter()
                .filter(|f| f.selected)
                .cloned()
                .collect(),
            OperationMode::Decrypt => self.file_manager.right_files.iter()
                .filter(|f| f.selected)
                .cloned()
                .collect(),
        };

        // 创建进度回调
        let progress_callback: ProgressCallback = {
            // 注意：这里我们不能直接捕获self，因为会导致借用检查器问题
            // 在实际应用中，可能需要使用消息传递或其他机制来更新UI
            Arc::new(move |progress_info: ProgressInfo| {
                // 这里可以发送消息到主线程更新UI
                println!("Progress: {}/{} files, current: {} ({:.1}%)",
                    progress_info.current_file_index + 1,
                    progress_info.total_files,
                    progress_info.current_file,
                    progress_info.overall_progress * 100.0
                );
            })
        };

        // Start async crypto operation
        match CryptoEngine::start_operation_async_static(
            self.settings.clone(),
            selected_files,
            Some(progress_callback),
        ) {
            Ok(handle) => {
                self.operation_handle = Some(handle);
                // 操作已启动，状态保持为Running
            }
            Err(error) => {
                self.dialog.error_message = error;
                self.dialog.show_error_dialog = true;
                self.app_state = AppState::Idle;
            }
        }
    }
    
    fn stop_operation(&mut self) {
        if let Some(handle) = &self.operation_handle {
            handle.stop();
        }
        self.operation_handle = None;
        self.app_state = AppState::Idle;
        self.progress.current_progress = 0.0;
        self.progress.total_progress = 0.0;
        self.progress.current_file_name = String::new();
    }

    fn resume_operation(&mut self) {
        self.app_state = AppState::Running;
    }

    fn skip_current_task(&mut self) {
        if let Some(handle) = &self.operation_handle {
            handle.skip_current();
        }
        self.progress.current_progress = 0.0;
    }

    /// 检查异步操作状态并更新UI
    fn check_operation_status(&mut self) {
        if let Some(handle) = &mut self.operation_handle {
            // 尝试接收进度更新
            while let Some(progress_info) = handle.try_recv_progress() {
                self.progress.current_file_name = progress_info.current_file;
                self.progress.current_progress = progress_info.current_file_progress;
                self.progress.total_progress = progress_info.overall_progress;
                self.progress.current_file_index = progress_info.current_file_index;
                self.progress.total_files = progress_info.total_files;
                self.progress.current_file_size = progress_info.current_file_size;
                self.progress.processed_bytes = progress_info.processed_bytes;
                self.progress.total_bytes = progress_info.total_bytes;
                self.progress.speed_mbps = progress_info.speed_mbps;
                self.progress.elapsed_time = progress_info.elapsed_time;
                self.progress.estimated_remaining = progress_info.estimated_remaining;
            }

            // 检查操作是否完成
            if handle.is_finished() {
                let status = handle.status();
                match status {
                    crate::models::OperationStatus::Completed => {
                        self.dialog.show_complete_dialog = true;
                        self.app_state = AppState::Idle;
                    }
                    crate::models::OperationStatus::Failed(error) => {
                        self.dialog.error_message = error;
                        self.dialog.show_error_dialog = true;
                        self.app_state = AppState::Idle;
                    }
                    crate::models::OperationStatus::Cancelled => {
                        self.app_state = AppState::Idle;
                    }
                    _ => {}
                }

                // 清理操作句柄
                self.operation_handle = None;
            }
        }
    }
    
    fn select_left_directory(&mut self) {
        if let Some(path) = FileDialog::new()
            .set_title("Select Directory for Encryption")
            .pick_folder()
        {
            self.file_manager.left_directory = path.to_string_lossy().to_string();
            self.load_left_files();
        }
    }
    
    fn select_right_directory(&mut self) {
        if let Some(path) = FileDialog::new()
            .set_title("Select Directory for Decryption")
            .pick_folder()
        {
            self.file_manager.right_directory = path.to_string_lossy().to_string();
            self.load_right_files();
        }
    }
}

impl eframe::App for KryptonApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        // 检查异步操作状态
        self.check_operation_status();

        // 如果有正在进行的操作，请求持续重绘以更新进度
        if self.operation_handle.is_some() && self.app_state == AppState::Running {
            ctx.request_repaint();
        }

        egui::CentralPanel::default().show(ctx, |ui| {
            // Settings panel
            SettingsPanel::render(
                ui,
                &mut self.settings,
            );

            // ui.separator();

            
            // File panel
            if let Some(event) = FilePanel::render(
                ui,
                &mut self.file_manager,
                &self.settings,
            ) {
                match event {
                    PanelEvent::LoadLeftFiles => self.load_left_files(),
                    PanelEvent::LoadRightFiles => self.load_right_files(),
                    PanelEvent::SelectLeftDirectory => self.select_left_directory(),
                    PanelEvent::SelectRightDirectory => self.select_right_directory(),
                    _ => {}
                }
            }
            
            // Progress panel
            ProgressPanel::render(
                ui,
                &self.progress,
            );
            
            ui.separator();
            
            // Control panel
            if let Some(event) = ControlPanel::render(
                ui,
                &self.app_state,
            ) {
                match event {
                    PanelEvent::StartOperation => self.start_operation(),
                    PanelEvent::StopOperation => self.stop_operation(),
                    PanelEvent::ResumeOperation => self.resume_operation(),
                    _ => {}
                }
            }
        });
        
        // Render dialogs
        if let Some(event) = ErrorDialog::render(
            ctx,
            &mut self.dialog.show_error_dialog,
            &self.dialog.error_message,
        ) {
            match event {
                DialogEvent::SkipCurrentTask => self.skip_current_task(),
                DialogEvent::StopAllOperations => self.stop_operation(),
            }
        }
        
        CompleteDialog::render(
            ctx,
            &mut self.dialog.show_complete_dialog,
        );
    }
} 