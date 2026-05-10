use crate::emulator::memory::{MemoryRegion, ReadMemory, WriteMemory};

const TILES_START: usize = 0;
// Each tile is 16 bytes long, there are 3 sections, all containing 128 tiles each.
const TILES_END: usize = 16 * 128 * 3;

pub type VRamData = MemoryRegion<8192, 0x8000, 0x9FFF>;
#[derive(Clone, Debug)]
pub struct VRam {
    data: VRamData,
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
        if VRamData::address_to_idx(address) <= TILES_END {
            self.compute_tiles();
        }
    }
}

impl VRam {
    pub fn new() -> Self {
        let data = VRamData::default();
        let tiles = Vec::with_capacity(TILES_END - TILES_START);

        let mut this = Self {
            data,
            tiles
        };

        this.compute_tiles();
        this
    }

    fn compute_tiles(&mut self) {
        let bytes = self.data.0.get(0..(10*128*3)).unwrap();

        let is_initializing = self.tiles.is_empty();
        for (i, tile_bytes) in bytes.chunks(16).enumerate() {
            let colors = tile_bytes.chunks(2).into_iter().fold(Vec::new(), |mut colors, bytes| {
                let lsb = bytes[0];
                let msb = bytes[1];

                for i in (0..8).rev() {
                    let lsb_bit = (lsb >> i) & 1;
                    let msb_bit = (msb >> i) & 1;

                    colors.push(msb_bit << 1 | lsb_bit);
                }

                colors
            });

            let tile = Tile::new(colors);
            if is_initializing {
                self.tiles.push(tile);
            } else {
                self.tiles[i] = tile;
            }
        }
    }
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
