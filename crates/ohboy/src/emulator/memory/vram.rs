use crate::emulator::memory::{MemoryRegion, ReadMemory, WriteMemory};

const TILES_START: usize = 0;
// Each tile is 16 bytes long, there are 3 sections, all containing 128 tiles each.
const TILES_END: usize = 16 * 128 * 3;

pub type VRamData = MemoryRegion<8192, 0x8000, 0x9FFF>;
#[derive(Clone, Debug)]
pub struct VRam {
    data: VRamData,

    pub dirty_tiles: bool, 
    pub tiles: Vec<Tile>,
}

impl ReadMemory for VRam {
    fn read_memory(&self, address: u16) -> u8 {
        self.data.read_memory(address)
    }
}

impl WriteMemory for VRam {
    fn write_memory(&mut self, address: u16, value: u8) {
        self.data.write_memory(address, value);
        if VRamData::address_to_idx(address) - TILES_START < TILES_END {
            self.sync_tile(address);
        }
    }
}

impl VRam {
    pub fn new() -> Self {
        let data = VRamData::default();
        let tiles = Vec::with_capacity(TILES_END - TILES_START);

        let mut this = Self {
            data,

            dirty_tiles: false,
            tiles
        };

        this.init_tiles();
        this
    }

    fn init_tiles(&mut self) {
        self.tiles.clear();
        let bytes = self.data.0.get(0..TILES_END).unwrap();

        let is_initializing = self.tiles.is_empty();
        for tile_bytes in bytes.chunks(16) {
            let colors = tile_bytes.chunks(2).into_iter().fold(Vec::with_capacity(64), |mut colors, pair| {
                let idxs = byte_pair_to_idxs(pair);
                colors.extend_from_slice(&idxs);

                colors
            });

            let tile = Tile::new(colors);
            if is_initializing {
                self.tiles.push(tile);
            }
        }
        self.dirty_tiles = true;
    }

    fn sync_tile(&mut self, address: u16) {
        let actual_idx = VRamData::address_to_idx(address) - TILES_START; // idx of modified data inside the vram data struct
        let tile_idx = (actual_idx) / 16; // idx of the tile
        let idx = (actual_idx) % 16; // idx of the affected
        

        let tile = self.tiles.get_mut(tile_idx).unwrap();

        let row_to_replace = idx / 2;

        let pair_idx = actual_idx - (actual_idx % 2);
        let byte_pair = self.data.0.get(pair_idx..=(pair_idx+1)).unwrap();
        tile.color_indexes[row_to_replace*8..(row_to_replace*8 + 8)].copy_from_slice(&byte_pair_to_idxs(byte_pair));
        self.dirty_tiles = true;
    }
}

fn byte_pair_to_idxs(pair: &[u8]) -> [u8; 8] {
    let lsb = pair[0];
    let msb = pair[1];

    let mut colors = [0; 8];
    for i in 0..8 {
        let shift = 7 - i;
        let lsb_bit = (lsb >> shift) & 1;
        let msb_bit = (msb >> shift) & 1;

        colors[i] = msb_bit << 1 | lsb_bit;
    }

    colors
}

#[derive(Clone, Debug)]
pub struct Tile {
    pub color_indexes: Vec<u8>,
}

impl Tile {
    pub fn new(color_indexes: Vec<u8>) -> Self {
        Self { color_indexes }
    }
}
