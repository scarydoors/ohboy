use egui::{FontFamily, FontId, RichText, TextureId, epaint::TextureManager};

use crate::emulator::{self, Snapshot};

pub struct UiState {
    snapshot: Option<Snapshot>,
    texture_manager: TextureManager,
    tile_textures: Vec<TextureId>
    // use texture manager somehow here
}

impl UiState {
    pub fn new() -> Self {
        UiState {
            snapshot: None,
            texture_manager: TextureManager::default(),
            tile_textures: Vec::with_capacity(3 * 128)
        }
    }

    pub fn update(snapshot: Snapshot) {

    }
}

pub fn render(ctx: &egui::Context, state: UiState) {
    if let Some(snapshot) = state.snapshot {
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

        egui::Window::new("Tile viewer")
            .default_open(true)
            .resizable(true)
            .show(ctx, |ui| {
                egui::Image::new() 
            });
    }
}
