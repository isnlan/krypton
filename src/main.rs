use eframe::egui;
use std::path::PathBuf;

#[derive(Debug, Clone, PartialEq)]
enum OperationMode {
    Encrypt,
    Decrypt,
}

#[derive(Debug, Clone, PartialEq)]
enum EncryptionAlgorithm {
    AES256,
    ChaCha20,
    Blowfish,
}

#[derive(Debug, Clone)]
struct FileItem {
    path: PathBuf,
    selected: bool,
    name: String,
}

#[derive(Debug, Clone, PartialEq)]
enum AppState {
    Idle,
    Running,
    Paused,
}

struct MyApp {
    // 设置区域
    operation_mode: OperationMode,
    encryption_algorithm: EncryptionAlgorithm,
    password: String,
    max_threads: u32,
    encrypt_filename: bool,
    delete_source: bool,
    
    // 文件预览区域
    left_directory: String,
    right_directory: String,
    left_files: Vec<FileItem>,
    right_files: Vec<FileItem>,
    
    // 进度区域
    current_progress: f32,
    total_progress: f32,
    current_file_name: String,
    
    // 应用状态
    app_state: AppState,
    
    // 弹出框状态
    show_error_dialog: bool,
    show_complete_dialog: bool,
    error_message: String,
}

impl Default for MyApp {
    fn default() -> Self {
        Self {
            operation_mode: OperationMode::Encrypt,
            encryption_algorithm: EncryptionAlgorithm::AES256,
            password: String::new(),
            max_threads: 1,
            encrypt_filename: true,
            delete_source: true,
            
            left_directory: String::new(),
            right_directory: String::new(),
            left_files: Vec::new(),
            right_files: Vec::new(),
            
            current_progress: 0.0,
            total_progress: 0.0,
            current_file_name: String::new(),
            
            app_state: AppState::Idle,
            
            show_error_dialog: false,
            show_complete_dialog: false,
            error_message: String::new(),
        }
    }
}

impl MyApp {
    fn render_settings_panel(&mut self, ui: &mut egui::Ui) {
        ui.group(|ui| {
            
            // First row: Operation mode, encryption algorithm, password input
            ui.horizontal(|ui| {
                // Operation mode selection
                ui.label("Mode: ");
                ui.radio_value(&mut self.operation_mode, OperationMode::Encrypt, "Encrypt");
                ui.radio_value(&mut self.operation_mode, OperationMode::Decrypt, "Decrypt");
                
                ui.separator();
                
                // Encryption algorithm selection
                ui.label("Algorithm: ");
                egui::ComboBox::from_label("")
                    .selected_text(format!("{:?}", self.encryption_algorithm))
                    .show_ui(ui, |ui| {
                        ui.selectable_value(&mut self.encryption_algorithm, EncryptionAlgorithm::AES256, "AES-256");
                        ui.selectable_value(&mut self.encryption_algorithm, EncryptionAlgorithm::ChaCha20, "ChaCha20");
                        ui.selectable_value(&mut self.encryption_algorithm, EncryptionAlgorithm::Blowfish, "Blowfish");
                    });
                
                ui.separator();
                
                // Key input
                ui.label("Key: ");
                ui.text_edit_singleline(&mut self.password);
            });
            
            // Second row: Max threads, checkboxes
            ui.horizontal(|ui| {
                // Max threads
                ui.label("Max Threads: ");
                ui.add(egui::Slider::new(&mut self.max_threads, 1..=16));
                
                ui.separator();
                
                // Checkboxes
                ui.checkbox(&mut self.encrypt_filename, "Encrypt Filename");
                ui.checkbox(&mut self.delete_source, "Delete Source");
            });
        });
    }
    
    fn render_file_panel(&mut self, ui: &mut egui::Ui) {
        ui.horizontal(|ui| {
            // Left file preview area
            ui.group(|ui| {
                ui.set_min_width(300.0);
                ui.label("Files to Encrypt");
                ui.separator();
                
                // Directory selection
                ui.horizontal(|ui| {
                    ui.text_edit_singleline(&mut self.left_directory);
                    if ui.button("Open Directory").clicked() {
                        // File dialog should be called here
                        self.load_left_files();
                    }
                });
                
                // File list
                egui::ScrollArea::vertical()
                    .id_salt("left_files_scroll")
                    .max_height(200.0)
                    .show(ui, |ui| {
                        for file in &mut self.left_files {
                            ui.horizontal(|ui| {
                                ui.checkbox(&mut file.selected, "");
                                ui.label(&file.name);
                            });
                        }
                    });
            });
            
            // Right file preview area
            ui.group(|ui| {
                ui.set_min_width(300.0);
                ui.label("Files to Decrypt");
                ui.separator();
                
                // Directory selection
                ui.horizontal(|ui| {
                    ui.text_edit_singleline(&mut self.right_directory);
                    if ui.button("Open Directory").clicked() {
                        // File dialog should be called here
                        self.load_right_files();
                    }
                });
                
                // File list
                egui::ScrollArea::vertical()
                    .id_salt("right_files_scroll")
                    .max_height(200.0)
                    .show(ui, |ui| {
                        for file in &mut self.right_files {
                            ui.horizontal(|ui| {
                                ui.checkbox(&mut file.selected, "");
                                ui.label(&file.name);
                            });
                        }
                    });
            });
        });
    }
    
    fn render_progress_panel(&mut self, ui: &mut egui::Ui) {
        ui.group(|ui| {
            ui.label("Progress");
            ui.separator();
            
            // Current file progress
            ui.horizontal(|ui| {
                ui.label("Current File: ");
                ui.label(&self.current_file_name);
            });
            ui.add(egui::ProgressBar::new(self.current_progress).text("Current Progress"));
            
            ui.separator();
            
            // Overall progress
            ui.label("Overall Progress: ");
            ui.add(egui::ProgressBar::new(self.total_progress).text("Total Progress"));
        });
    }
    
    fn render_control_panel(&mut self, ui: &mut egui::Ui) {
        ui.horizontal(|ui| {
            if ui.button("Exit").clicked() {
                std::process::exit(0);
            }
            
            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                match self.app_state {
                    AppState::Idle => {
                        if ui.button("Start").clicked() {
                            self.start_operation();
                        }
                    }
                    AppState::Running => {
                        if ui.button("Stop").clicked() {
                            self.stop_operation();
                        }
                    }
                    AppState::Paused => {
                        if ui.button("Resume").clicked() {
                            self.resume_operation();
                        }
                    }
                }
            });
        });
    }
    
    fn render_dialogs(&mut self, ctx: &egui::Context) {
        // Error dialog
        if self.show_error_dialog {
            egui::Window::new("Error")
                .collapsible(false)
                .resizable(false)
                .show(ctx, |ui| {
                    ui.label(&self.error_message);
                    ui.separator();
                    ui.horizontal(|ui| {
                        if ui.button("Skip").clicked() {
                            self.show_error_dialog = false;
                            self.skip_current_task();
                        }
                        if ui.button("Stop All").clicked() {
                            self.show_error_dialog = false;
                            self.stop_operation();
                        }
                    });
                });
        }
        
        // Complete dialog
        if self.show_complete_dialog {
            egui::Window::new("Complete")
                .collapsible(false)
                .resizable(false)
                .show(ctx, |ui| {
                    ui.label("Operation completed successfully!");
                    ui.separator();
                    if ui.button("OK").clicked() {
                        self.show_complete_dialog = false;
                    }
                });
        }
    }
    
    fn load_left_files(&mut self) {
        // 模拟加载文件列表
        self.left_files = vec![
            FileItem {
                path: PathBuf::from("document1.txt"),
                selected: false,
                name: "document1.txt".to_string(),
            },
            FileItem {
                path: PathBuf::from("image.jpg"),
                selected: false,
                name: "image.jpg".to_string(),
            },
        ];
    }
    
    fn load_right_files(&mut self) {
        // 模拟加载文件列表
        self.right_files = vec![
            FileItem {
                path: PathBuf::from("encrypted_file.enc"),
                selected: false,
                name: "encrypted_file.enc".to_string(),
            },
        ];
    }
    
    fn start_operation(&mut self) {
        self.app_state = AppState::Running;
        self.current_progress = 0.0;
        self.total_progress = 0.0;
        self.current_file_name = "Starting processing...".to_string();
        
        // Actual encryption/decryption operation should start here
        // For demonstration, we simulate an error
        self.error_message = "Demo error: Unable to access file".to_string();
        self.show_error_dialog = true;
    }
    
    fn stop_operation(&mut self) {
        self.app_state = AppState::Idle;
        self.current_progress = 0.0;
        self.total_progress = 0.0;
        self.current_file_name = String::new();
    }
    
    fn resume_operation(&mut self) {
        self.app_state = AppState::Running;
    }
    
    fn skip_current_task(&mut self) {
        // Skip current task and continue to next one
        self.current_progress = 0.0;
        // Skip logic should be implemented here
    }
}

impl eframe::App for MyApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        egui::CentralPanel::default().show(ctx, |ui| {
            // Settings area
            self.render_settings_panel(ui);
            
            // File preview area
            self.render_file_panel(ui);
            
            // Progress area
            self.render_progress_panel(ui);
            ui.separator();
            
            // Control panel
            self.render_control_panel(ui);
        });
        
        // 渲染对话框
        self.render_dialogs(ctx);
    }
}

fn main() -> Result<(), eframe::Error> {
    env_logger::init();
    
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([1000.0, 700.0])
            .with_title("File Encryption/Decryption Tool"),
        ..Default::default()
    };
    
    eframe::run_native(
        "Krypton - File Encryption Tool",
        options,
        Box::new(|_cc| {
            Ok(Box::new(MyApp::default()))
        }),
    )
}
