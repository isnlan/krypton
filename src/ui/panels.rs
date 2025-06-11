use eframe::egui;
use crate::models::{OperationMode, EncryptionAlgorithm, FileItem, AppState, Settings, FileManagerState, ProgressState};

#[derive(Debug, Clone, PartialEq)]
pub enum PanelEvent {
    LoadLeftFiles,
    LoadRightFiles,
    StartOperation,
    StopOperation,
    ResumeOperation,
    SelectLeftDirectory,
    SelectRightDirectory,
}

pub struct SettingsPanel;

impl SettingsPanel {
    pub fn render(
        ui: &mut egui::Ui,
        settings: &mut Settings,
    ) {
        ui.set_width(ui.available_width());
        
        // First row: Operation mode, encryption algorithm, password input
        ui.horizontal(|ui| {
            ui.set_width(ui.available_width());
            
            // Operation mode selection
            ui.label("Mode: ");
            ui.radio_value(&mut settings.operation_mode, OperationMode::Encrypt, "Encrypt");
            ui.radio_value(&mut settings.operation_mode, OperationMode::Decrypt, "Decrypt");
            
            ui.separator();
            
            // Encryption algorithm selection
            ui.label("Algorithm: ");
            egui::ComboBox::from_label("")
                .selected_text(settings.encryption_algorithm.to_string())
                .show_ui(ui, |ui| {
                    ui.selectable_value(&mut settings.encryption_algorithm, EncryptionAlgorithm::AES256, "AES-256");
                    ui.selectable_value(&mut settings.encryption_algorithm, EncryptionAlgorithm::ChaCha20, "ChaCha20");
                });
            
            ui.separator();
            
            // Key input - fixed width
            ui.label("Key: ");
            ui.add_sized(
                [400.0, 20.0],
                egui::TextEdit::singleline(&mut settings.password)
                    .frame(true)
            );
        });
        
        // Second row: Max threads, file extension, checkboxes
        ui.horizontal(|ui| {
            ui.set_width(ui.available_width());
            
            // Max threads - fixed width
            ui.label("Max Threads: ");
            ui.add_sized(
                [200.0, 20.0],
                egui::Slider::new(&mut settings.max_threads, 1..=16)
            );
            
            ui.separator();
            
            // File extension input - fixed width
            ui.label("File Extension: ");
            ui.add_sized(
                [100.0, 20.0],
                egui::TextEdit::singleline(&mut settings.file_extension)
                    .frame(true)
                    .hint_text("enc")
            );
            
            ui.separator();
            
            // Checkboxes - left aligned
            ui.checkbox(&mut settings.encrypt_filename, "Encrypt Filename");
            ui.checkbox(&mut settings.delete_source, "Delete Source");
        });
    }
}

pub struct FilePanel;

impl FilePanel {
    pub fn render(
        ui: &mut egui::Ui,
        file_manager: &mut FileManagerState,
        settings: &Settings,
    ) -> Option<PanelEvent> {
        let mut event = None;
        
        // Wrap entire FilePanel in a group with fixed height
        ui.group(|ui| {
            ui.set_height(400.0);
            ui.horizontal(|ui| {
                // Left file preview area
                ui.allocate_ui_with_layout(
                    egui::Vec2::new(ui.available_width() / 2.0 - 1.0, ui.available_height()),
                    egui::Layout::top_down(egui::Align::LEFT),
                    |ui| {
                        ui.horizontal(|ui| {
                            ui.label("Files to Encrypt");
                            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                                ui.label(format!("({} files)", file_manager.left_files.len()));
                            });
                        });
                        ui.separator();
                        
                        // Directory selection
                        ui.horizontal(|ui| {
                            let text_edit_response = ui.add(
                                egui::TextEdit::singleline(&mut file_manager.left_directory)
                                    .frame(true)
                            );
                            
                            // 当用户完成编辑（失去焦点且内容改变）时自动加载文件
                            if text_edit_response.lost_focus() && !file_manager.left_directory.is_empty() {
                                event = Some(PanelEvent::LoadLeftFiles);
                            }
                            
                            if ui.button("Open Directory").clicked() {
                                event = Some(PanelEvent::SelectLeftDirectory);
                            }
                            
                            // 添加刷新按钮
                            if ui.button("Refresh").clicked() && !file_manager.left_directory.is_empty() {
                                event = Some(PanelEvent::LoadLeftFiles);
                            }
                        });
                        
                        // File list - occupy remaining height
                        let remaining_height = (ui.available_height() - 10.0).max(400.0); // 确保最小高度
                        ui.group(|ui| {
                            egui::ScrollArea::vertical()
                                .id_salt("left_files_scroll")
                                .auto_shrink([false, false])
                                .min_scrolled_height(remaining_height)
                                .max_height(remaining_height)
                                .show(ui, |ui| {
                                    ui.set_min_height(remaining_height);
                                    for (index, file) in file_manager.left_files.iter_mut().enumerate() {
                                        ui.horizontal(|ui| {
                                            ui.checkbox(&mut file.selected, "");
                                            ui.label(format!("{}. {}", index + 1, &file.name));
                                        });
                                    }
                                    // 如果没有文件，显示提示信息
                                    if file_manager.left_files.is_empty() && !file_manager.left_directory.is_empty() {
                                        ui.centered_and_justified(|ui| {
                                            ui.label("No files found in this directory");
                                        });
                                    } else if file_manager.left_files.is_empty() {
                                        ui.centered_and_justified(|ui| {
                                            ui.label("Select a directory to see files");
                                        });
                                    }
                                });
                        });
                    }
                );
                
                // Vertical separator line
                ui.separator();
                
                // Right file preview area
                ui.allocate_ui_with_layout(
                    egui::Vec2::new(ui.available_width(), ui.available_height()),
                    egui::Layout::top_down(egui::Align::LEFT),
                    |ui| {
                        ui.horizontal(|ui| {
                            ui.label("Files to Decrypt");
                            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                                ui.label(format!("({} files)", file_manager.right_files.len()));
                            });
                        });
                        ui.separator();
                        
                        // Directory selection
                        ui.horizontal(|ui| {
                            let text_edit_response = ui.add(
                                egui::TextEdit::singleline(&mut file_manager.right_directory)
                                    .frame(true)
                            );
                            
                            // 当用户完成编辑（失去焦点且内容改变）时自动加载文件
                            if text_edit_response.lost_focus() && !file_manager.right_directory.is_empty() {
                                event = Some(PanelEvent::LoadRightFiles);
                            }
                            
                            if ui.button("Open Directory").clicked() {
                                event = Some(PanelEvent::SelectRightDirectory);
                            }
                            
                            // 添加刷新按钮
                            if ui.button("Refresh").clicked() && !file_manager.right_directory.is_empty() {
                                event = Some(PanelEvent::LoadRightFiles);
                            }
                        });
                        
                        // File list - occupy remaining height
                        let remaining_height = (ui.available_height() - 10.0).max(400.0); // 确保最小高度
                        ui.group(|ui| {
                            egui::ScrollArea::vertical()
                                .id_salt("right_files_scroll")
                                .auto_shrink([false, false])
                                .min_scrolled_height(remaining_height)
                                .max_height(remaining_height)
                                .show(ui, |ui| {
                                    ui.set_min_height(remaining_height);
                                    for (index, file) in file_manager.right_files.iter_mut().enumerate() {
                                        ui.horizontal(|ui| {
                                            ui.checkbox(&mut file.selected, "");
                                            ui.label(format!("{}. {}", index + 1, &file.name));
                                        });
                                    }
                                    // 如果没有文件，显示提示信息
                                    if file_manager.right_files.is_empty() && !file_manager.right_directory.is_empty() {
                                        ui.centered_and_justified(|ui| {
                                            ui.label(format!("No .{} files found in this directory", settings.file_extension));
                                        });
                                    } else if file_manager.right_files.is_empty() {
                                        ui.centered_and_justified(|ui| {
                                            ui.label("Select a directory to see encrypted files");
                                        });
                                    }
                                });
                        });
                    }
                );
            });
        });
        
        event
    }
}

pub struct ProgressPanel;

impl ProgressPanel {
    pub fn render(
        ui: &mut egui::Ui,
        progress: &ProgressState,
    ) {
        ui.group(|ui| {
            ui.label("Progress");
            ui.separator();
            
            // Current file progress
            ui.horizontal(|ui| {
                ui.label("Current File: ");
                ui.label(&progress.current_file_name);
            });
            ui.add(egui::ProgressBar::new(progress.current_progress).text("Current Progress"));
            
            ui.separator();
            
            // Overall progress
            ui.label("Overall Progress: ");
            ui.add(egui::ProgressBar::new(progress.total_progress).text("Total Progress"));
        });
    }
}

pub struct ControlPanel;

impl ControlPanel {
    pub fn render(
        ui: &mut egui::Ui,
        app_state: &AppState,
    ) -> Option<PanelEvent> {
        let mut event = None;
        ui.horizontal(|ui| {
            if ui.button("Exit").clicked() {
                std::process::exit(0);
            }
            
            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                match app_state {
                    AppState::Idle => {
                        if ui.button("Start").clicked() {
                            event = Some(PanelEvent::StartOperation);
                        }
                    }
                    AppState::Running => {
                        if ui.button("Stop").clicked() {
                            event = Some(PanelEvent::StopOperation);
                        }
                    }
                    AppState::Paused => {
                        if ui.button("Resume").clicked() {
                            event = Some(PanelEvent::ResumeOperation);
                        }
                    }
                }
            });
        });
        
        event
    }
} 