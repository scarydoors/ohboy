use egui::{FontFamily, FontId, RichText};

use crate::emulator::cpu::registers::Registers;

pub fn render(ctx: &egui::Context) {
    let example = Registers::new();
    egui::Window::new("Registers")
        .default_open(true)
        .resizable(false)
        .show(ctx, |ui| {
            egui::Grid::new("registers")
                .striped(true)
                .show(ui, |ui| {
                    ui.horizontal(|ui| {
                        ui.label("PC:");
                        ui.label(RichText::new(format!("{:#06x}", example.pc.get())).font(FontId::monospace(14.0)));
                    });
                    ui.horizontal(|ui| {
                        ui.label("SP:");
                        ui.label(RichText::new(format!("{:#06x}", example.sp.get())).font(FontId::monospace(14.0)));
                    });
                    ui.end_row();
                    ui.horizontal(|ui| {
                        ui.label("A:");
                        ui.label(RichText::new(format!("{:#04x}", example.a.get())).font(FontId::monospace(14.0)));
                    });
                    ui.horizontal(|ui| {
                        ui.label("F:");
                        ui.label(RichText::new(format!("{:#04x}", example.f.get())).font(FontId::monospace(14.0)));
                    });
                    ui.end_row();
                })
    });
}
