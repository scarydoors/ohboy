use egui::{FontFamily, FontId, ImageSource, RichText, TextureHandle, TextureOptions, load::SizedTexture};

use crate::emulator::{self, Snapshot};

pub struct UiState {
    snapshot: Option<Snapshot>,
    tile_textures: Vec<TextureHandle>
}

impl UiState {
    pub fn new() -> Self {
        UiState {
            snapshot: None,
            tile_textures: Vec::with_capacity(3 * 128)
        }
    }

    pub fn update(&mut self, ctx: &egui::Context, snapshot: Snapshot) {
        if self.tile_textures.is_empty() {
            for (i, tile) in snapshot.vram.tiles.iter().enumerate() {
                // TODO: this could be a method on the tile
                let rgb_data: Vec<u8> = tile.color_indexes.iter().copied().flat_map(|i| idx_to_rgb(i)).collect();
                self.tile_textures.push(
                    ctx.load_texture(format!("Tile {i}"), egui::ColorImage::from_rgb([8, 8], &rgb_data), TextureOptions::NEAREST)
                ); 
            }
        } else {
            for (i, tile) in snapshot.vram.tiles.iter().enumerate() {
                let rgb_data: Vec<u8> = tile.color_indexes.iter().copied().flat_map(|i| idx_to_rgb(i)).collect();
                self.tile_textures[i].set(egui::ColorImage::from_rgb([8, 8], &rgb_data), TextureOptions::NEAREST)
            }
        }

        self.snapshot = Some(snapshot);
    }
}

pub fn idx_to_rgb(idx: u8) -> [u8; 3] {
    match idx {
        3 => [0, 0, 0],
        2 => [85, 85, 85],
        1 => [170, 170, 170],
        0 => [255, 255, 255],
        _ => panic!("wat"),
    }
}

pub fn render(ctx: &egui::Context, state: &UiState) {
    if let Some(snapshot) = &state.snapshot {
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
                ui.image(ImageSource::Texture(SizedTexture::from_handle(&state.tile_textures[0])))
            });
    }
}
