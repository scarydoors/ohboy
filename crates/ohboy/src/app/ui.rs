use egui::{FontFamily, FontId, RichText};

use crate::emulator;

pub struct UiState {
    emulator_snapshot: Option<emulator::Snapshot>,
}

pub fn render(ctx: &egui::Context, state: UiState) {
    if let Some(snapshot) = snapshot {
        let registers = &snapshot.registers;
        egui::Window::new("Registers")
            .default_open(true)
            .resizable(false)
            .show(ctx, |ui| {
                egui::Grid::new("registers")
                    .striped(true)
                    .show(ui, |ui| {
                        ui.horizontal(|ui| {
                            ui.label("PC:");
                            ui.label(RichText::new(format!("{:#06x}", registers.pc.get())).font(FontId::monospace(14.0)));
                        });
                        ui.horizontal(|ui| {
                            ui.label("SP:");
                            ui.label(RichText::new(format!("{:#06x}", registers.sp.get())).font(FontId::monospace(14.0)));
                        });
                        ui.end_row();
                        ui.horizontal(|ui| {
                            ui.label("A:");
                            ui.label(RichText::new(format!("{:#04x}", registers.a.get())).font(FontId::monospace(14.0)));
                        });
                        ui.horizontal(|ui| {
                            ui.label("F:");
                            ui.label(RichText::new(format!("{:#04x}", registers.f.get())).font(FontId::monospace(14.0)));
                        });
                        ui.end_row();
                    })
            });
    }
}
