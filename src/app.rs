use eframe::egui;

pub struct ImgProcDemo {

}

impl ImgProcDemo {
    pub fn new(_cc: &eframe::CreationContext<'_>) -> Self {
        Self {  }
    }
}

impl eframe::App for ImgProcDemo {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        egui::CentralPanel::default().show(ctx, |ui| {
            ui.heading("Image Processing Demo");
            ui.label("hi");
        });
    }
}

