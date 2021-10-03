use std::collections::HashMap;

use quartz_nbt::{NbtCompound, NbtList, NbtTag};

use crate::{volume::Volume, BlockState, RegionParseError, Vector3};

/// Represents a region of blocks
pub struct Region {
    /// The original volume taken up by the region
    ///
    /// May not reflect the actual space taken up when new blocks are inserted into the region. For that, use the [volume()](Region::volume) method
    pub volume: Volume,
    pub(super) blocks: HashMap<Vector3<i32>, BlockState>,
    pub(super) entities: Option<NbtList>,
    pub(super) pending_block_ticks: Option<NbtList>,
    pub(super) pending_fluid_ticks: Option<NbtList>,
    pub(super) tile_entities: Option<NbtList>,
}

// https://github.com/maruohon/litematica/issues/53#issuecomment-520281566
impl Region {
    /// Create a new region
    pub fn new() -> Region {
        Region {
            volume: Volume::default(),
            blocks: HashMap::new(),
            entities: None,
            pending_block_ticks: None,
            pending_fluid_ticks: None,
            tile_entities: None,
        }
    }

    /// Calculates the volume taken up by a region including all the blocks in it
    pub fn volume(&self) -> Volume {
        self.blocks.keys().fold(self.volume, |volume, value| {
            volume.expand_to_fit(*value + self.volume.origin())
        })
    }

    /// Set a block state in the region
    pub fn set_block(&mut self, pos: Vector3<i32>, block: BlockState) {
        if block != BlockState::new("air", None) {
            self.blocks.insert(pos, block);
        } else {
            self.blocks.remove(&pos);
        }
    }

    /// A read only map of all the blocks in the region, excluding air blocks
    pub fn blocks(&self) -> &HashMap<Vector3<i32>, BlockState> {
        &self.blocks
    }

    pub(crate) fn new_from_nbt(data: NbtCompound) -> Result<Region, RegionParseError> {
        let palette =
            if let NbtTag::List(palette_list) = data.get::<_, &NbtTag>("BlockStatePalette")? {
                palette_list
            } else {
                return Err(RegionParseError::WrongTag("BlockStatePalette".to_string()));
            };

        let parsed_palette = Region::parse_palette(palette)?;

        let blocks_long_array =
            if let NbtTag::LongArray(array) = data.get::<_, &NbtTag>("BlockStates")? {
                array
            } else {
                return Err(RegionParseError::WrongTag("BlockStates".to_string()));
            };

        let size = Vector3::from_nbt(&data, "Size")?;

        Ok(Region {
            volume: Volume::new(Vector3::from_nbt(&data, "Position")?, size),
            blocks: Region::unpack_packed_array(
                blocks_long_array,
                &parsed_palette,
                Region::calculate_bits(parsed_palette.len()),
                size,
            ),
            entities: data.get::<_, &NbtList>("Entities").ok().map(|v| v.clone()),
            pending_block_ticks: data
                .get::<_, &NbtList>("PendingBlockTicks")
                .ok()
                .map(|v| v.clone()),
            pending_fluid_ticks: data
                .get::<_, &NbtList>("PendingFluidTicks")
                .ok()
                .map(|v| v.clone()),
            tile_entities: data
                .get::<_, &NbtList>("TileEntities")
                .ok()
                .map(|v| v.clone()),
        })
    }

    pub(crate) fn to_nbt(&self) -> (NbtCompound, Volume) {
        let mut out = NbtCompound::new();

        let palette = Region::generate_palette_nbt(&self.blocks);

        out.insert(
            "BlockStatePalette",
            palette.iter().fold(NbtList::new(), |mut a, v| {
                a.push(v);
                a
            }),
        );

        self.write_misc_data(&mut out);

        let volume = self.volume();

        out.insert("Position", volume.origin());
        out.insert("Size", volume.size());

        out.insert(
            "BlockStates",
            self.generate_block_states_nbt(volume.make_size_positive(), &palette),
        );

        (out, volume)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_from_nbt() {
        let mut root = NbtCompound::new();

        let mut palette = NbtList::new();

        palette.push(&BlockState::new("bruh", None));
        palette.push(&BlockState::new("yeet", None));
        palette.push(&BlockState::new("poggers?", None));
        palette.push(&BlockState::new("poggers.", None));

        root.insert("BlockStatePalette", palette);

        root.insert(
            "BlockStates",
            vec![0x0123456789abcdef_i64, 0x1032547698badcfe],
        );

        root.insert("Size", Vector3::new(4, 4, 4));
        root.insert("Position", Vector3::new(0, 0, 0));

        Region::new_from_nbt(root).unwrap();
    }
}
