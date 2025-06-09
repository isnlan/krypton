use eframe::egui;
use crate::models::{OperationMode, EncryptionAlgorithm, FileItem, AppState, Settings, FileManagerState, ProgressState};

#[derive(Debug, Clone, PartialEq)]
pub enum PanelEvent {
    LoadLeftFiles,
    LoadRightFiles,
    StartOperation,
    StopOperation,
    ResumeOperation,
}

pub struct SettingsPanel;

impl SettingsPanel {
    pub fn render(
        ui: &mut egui::Ui,
        settings: &mut Settings,
    ) {
        ui.group(|ui| {
            ui.set_width(ui.available_width());
            ui.label("Settings");
            ui.separator();
            
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
                        ui.selectable_value(&mut settings.encryption_algorithm, EncryptionAlgorithm::Blowfish, "Blowfish");
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
            
            // Second row: Max threads, checkboxes
            ui.horizontal(|ui| {
                ui.set_width(ui.available_width());
                
                // Max threads - fixed width
                ui.label("Max Threads: ");
                ui.add_sized(
                    [200.0, 20.0],
                    egui::Slider::new(&mut settings.max_threads, 1..=16)
                );
                
                ui.separator();
                
                // Checkboxes - left aligned
                ui.checkbox(&mut settings.encrypt_filename, "Encrypt Filename");
                ui.checkbox(&mut settings.delete_source, "Delete Source");
            });
        });
    }
}

pub struct FilePanel;

impl FilePanel {
    pub fn render(
        ui: &mut egui::Ui,
        file_manager: &mut FileManagerState,
    ) -> Option<PanelEvent> {
        let mut event = None;
        ui.columns(2, |columns| {
            // Left file preview area
            columns[0].group(|ui| {
                ui.label("Files to Encrypt");
                ui.separator();
                
                // Directory selection
                ui.horizontal(|ui| {
                    ui.add(egui::TextEdit::singleline(&mut file_manager.left_directory)
                        .frame(true));
                    if ui.button("Open Directory").clicked() {
                        event = Some(PanelEvent::LoadLeftFiles);
                    }
                });
                
                // File list - fixed height 200 pixels
                let (_, file_list_rect) = ui.allocate_space([ui.available_width(), 200.0].into());
                ui.allocate_ui_at_rect(file_list_rect, |ui| {
                    egui::ScrollArea::vertical()
                        .id_salt("left_files_scroll")
                        .show(ui, |ui| {
                            for file in file_manager.left_files.iter_mut() {
                                ui.horizontal(|ui| {
                                    ui.checkbox(&mut file.selected, "");
                                    ui.label(&file.name);
                                });
                            }
                        });
                });
            });
            
            // Right file preview area
            columns[1].group(|ui| {
                ui.label("Files to Decrypt");
                ui.separator();
                
                // Directory selection
                ui.horizontal(|ui| {
                    ui.add(egui::TextEdit::singleline(&mut file_manager.right_directory)
                        .frame(true));
                    if ui.button("Open Directory").clicked() {
                        event = Some(PanelEvent::LoadRightFiles);
                    }
                });
                
                // File list - fixed height 200 pixels
                let (_, file_list_rect) = ui.allocate_space([ui.available_width(), 200.0].into());
                ui.allocate_ui_at_rect(file_list_rect, |ui| {
                    egui::ScrollArea::vertical()
                        .id_salt("right_files_scroll")
                        .show(ui, |ui| {
                            for file in file_manager.right_files.iter_mut() {
                                ui.horizontal(|ui| {
                                    ui.checkbox(&mut file.selected, "");
                                    ui.label(&file.name);
                                });
                            }
                        });
                });
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