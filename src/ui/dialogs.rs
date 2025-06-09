use eframe::egui;

#[derive(Debug, Clone, PartialEq)]
pub enum DialogEvent {
    SkipCurrentTask,
    StopAllOperations,
}

pub struct ErrorDialog;

impl ErrorDialog {
    pub fn render(
        ctx: &egui::Context,
        show: &mut bool,
        error_message: &str,
    ) -> Option<DialogEvent> {
        let mut event = None;
        if *show {
            egui::Window::new("Error")
                .collapsible(false)
                .resizable(false)
                .show(ctx, |ui| {
                    ui.label(error_message);
                    ui.separator();
                    ui.horizontal(|ui| {
                        if ui.button("Skip").clicked() {
                            *show = false;
                            event = Some(DialogEvent::SkipCurrentTask);
                        }
                        if ui.button("Stop All").clicked() {
                            *show = false;
                            event = Some(DialogEvent::StopAllOperations);
                        }
                    });
                });
        }
        
        event
    }
}

pub struct CompleteDialog;

impl CompleteDialog {
    pub fn render(
        ctx: &egui::Context,
        show: &mut bool,
    ) {
        if *show {
            egui::Window::new("Complete")
                .collapsible(false)
                .resizable(false)
                .show(ctx, |ui| {
                    ui.label("Operation completed successfully!");
                    ui.separator();
                    if ui.button("OK").clicked() {
                        *show = false;
                    }
                });
        }
    }
} 