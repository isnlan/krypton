use eframe::egui;
use crate::models::{OperationMode, EncryptionAlgorithm, FileItem, AppState, Settings, FileManagerState, ProgressState, DialogState};
use crate::core::FileManager;
use crate::crypto::CryptoEngine;
use crate::ui::{SettingsPanel, FilePanel, ProgressPanel, ControlPanel, ErrorDialog, CompleteDialog, PanelEvent, DialogEvent};
use rfd::FileDialog;

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
}

impl Default for KryptonApp {
    fn default() -> Self {
        Self {
            settings: Settings::default(),
            file_manager: FileManagerState::default(),
            progress: ProgressState::default(),
            app_state: AppState::Idle,
            dialog: DialogState::default(),
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
        
        // Start crypto operation
        match CryptoEngine::start_operation(
            &self.settings,
            &selected_files,
        ) {
            Ok(_) => {
                self.dialog.show_complete_dialog = true;
                self.app_state = AppState::Idle;
            }
            Err(error) => {
                self.dialog.error_message = error;
                self.dialog.show_error_dialog = true;
            }
        }
    }
    
    fn stop_operation(&mut self) {
        CryptoEngine::stop_operation();
        self.app_state = AppState::Idle;
        self.progress.current_progress = 0.0;
        self.progress.total_progress = 0.0;
        self.progress.current_file_name = String::new();
    }
    
    fn resume_operation(&mut self) {
        self.app_state = AppState::Running;
    }
    
    fn skip_current_task(&mut self) {
        CryptoEngine::skip_current_task();
        self.progress.current_progress = 0.0;
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