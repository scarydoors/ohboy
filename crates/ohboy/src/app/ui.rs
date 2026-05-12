use egui::{FontFamily, FontId, ImageSource, RichText, TextureHandle, TextureOptions, Vec2, load::SizedTexture};

use crate::emulator::{self, cpu::registers::Registers};

pub struct UiState {
    tile_textures: Vec<TextureHandle>,
}

impl UiState {
    pub fn new() -> Self {
        UiState {
            tile_textures: Vec::with_capacity(3 * 128),
        }
    }

    pub fn update(&mut self, ctx: &egui::Context, state: &mut emulator::State) {
        if let Some(emulator) = &mut state.emulator {
            let vram = &mut emulator.memory.vram;

            if self.tile_textures.is_empty() {
                for (i, tile) in vram.tiles.iter().enumerate() {
                    // TODO: this could be a method on the tile
                    let rgb_data: Vec<u8> = tile.color_indexes.iter().copied().flat_map(|i| idx_to_rgb(i)).collect();
                    self.tile_textures.push(
                        ctx.load_texture(format!("Tile {i}"), egui::ColorImage::from_rgb([8, 8], &rgb_data), TextureOptions::NEAREST)
                    ); 
                }
            } else if vram.dirty_tiles {
                // FIXME: refreshes too often, should debounce this
                for (i, tile) in vram.tiles.iter().enumerate() {
                    let rgb_data: Vec<u8> = tile.color_indexes.iter().copied().flat_map(|i| idx_to_rgb(i)).collect();
                    self.tile_textures[i].set(egui::ColorImage::from_rgb([8, 8], &rgb_data), TextureOptions::NEAREST);
                }
            }

            vram.dirty_tiles = false;
        }
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
        egui::Window::new("Tile viewer")
            .default_open(true)
            .resizable(true)
            .vscroll(true)
            .show(ctx, |ui| {
                ui.horizontal_wrapped(|ui| {
                    ui.style_mut().spacing.item_spacing = Vec2::new(0.0, 0.0);
                    for tile in &state.tile_textures {
                        ui.add(egui::Image::new(ImageSource::Texture(SizedTexture::from_handle(tile))).fit_to_exact_size(Vec2::new(32.0, 32.0)));
                    }
                })
        });
}
