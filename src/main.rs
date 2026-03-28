use chrono::Local;
use eframe::{egui, NativeOptions};

fn main() -> eframe::Result<()> {
    let options = NativeOptions {
        viewport: egui::ViewportBuilder::default().with_inner_size([420.0, 220.0]),
        ..Default::default()
    };

    eframe::run_native(
        "Nostromo Dashboard",
        options,
        Box::new(|_cc| Ok(Box::new(DashboardApp::default()))),
    )
}

#[derive(Default)]
struct DashboardApp;

impl eframe::App for DashboardApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        // Continuously repaint so the clock stays current.
        ctx.request_repaint_after(std::time::Duration::from_secs(1));

        egui::CentralPanel::default().show(ctx, |ui| {
            ui.heading("Nostromo Dashboard");
            ui.separator();

            egui::Frame::group(ui.style()).show(ui, |ui| {
                ui.vertical_centered(|ui| {
                    ui.label(egui::RichText::new("Date & Time").strong());

                    let now = Local::now();
                    ui.add_space(8.0);
                    ui.label(
                        egui::RichText::new(now.format("%A, %B %d, %Y").to_string()).size(20.0),
                    );
                    ui.label(egui::RichText::new(now.format("%I:%M:%S %p").to_string()).size(28.0));
                });
            });
        });
    }
}
