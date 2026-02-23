use eframe::egui;

pub fn start_gui() {
    let options = eframe::NativeOptions::default();

    eframe::run_native(
        "Traclus Optimized",
        options,
        Box::new(|_cc| Box::new(MyApp)),
    )
    .unwrap();
}

struct MyApp;

impl eframe::App for MyApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        egui::CentralPanel::default().show(ctx, |ui| {
            ui.heading("Traclus Optimized Parallel Implementation");
        });
    }
}
