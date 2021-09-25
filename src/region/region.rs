use std::collections::HashMap;

use quartz_nbt::{NbtCompound, NbtList, NbtTag};

use crate::{vector::Volume, BlockState, IVector3, RegionParseError};

pub struct Region {
    pub position: IVector3,
    pub(super) block_state_palette: Vec<BlockState>,
    pub(super) blocks: HashMap<IVector3, usize>,
    pub(super) entities: Option<NbtList>,
    pub(super) pending_block_ticks: Option<NbtList>,
    pub(super) pending_fluid_ticks: Option<NbtList>,
    pub(super) tile_entities: Option<NbtList>,
}

impl Region {
    // https://github.com/maruohon/litematica/issues/53#issuecomment-520281566

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

        Ok(Region {
            position: IVector3::from_nbt(&data, "Position")?,
            blocks: Region::unpack_packed_array(
                blocks_long_array,
                Region::calculate_bits(parsed_palette.len()),
                IVector3::from_nbt(&data, "Size")?,
            ),
            block_state_palette: parsed_palette,
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

        let palette = self.generate_palette_nbt();

        out.insert("BlockStatePalette", palette);

        self.write_misc_data(&mut out);

        let volume = self.volume();

        out.insert("Position", volume.origin());
        out.insert("Size", volume.size());

        out.insert("BlockStates", self.generate_block_states_nbt(volume));

        (out, volume)
    }

    pub fn total_blocks(&self) -> i32 {
        self.blocks.len() as i32
    }

    pub fn volume(&self) -> Volume {
        self.blocks
            .keys()
            .fold(None, |maybe_volume: Option<Volume>, value| {
                Some(match maybe_volume {
                    None => Volume::new(*value, IVector3::new(1, 1, 1)),
                    Some(volume) => volume.expand_to_fit(*value),
                })
            })
            .unwrap_or(Volume::new(self.position, IVector3::new(0, 0, 0)))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_from_nbt() {
        let mut root = NbtCompound::new();

        let mut palette = NbtList::new();

        palette.push(BlockState::new("bruh", None).to_nbt());
        palette.push(BlockState::new("yeet", None).to_nbt());
        palette.push(BlockState::new("poggers?", None).to_nbt());
        palette.push(BlockState::new("poggers.", None).to_nbt());

        root.insert("BlockStatePalette", palette);

        root.insert(
            "BlockStates",
            vec![0x0123456789abcdef_i64, 0x1032547698badcfe],
        );

        root.insert("Size", IVector3::new(4, 4, 4));
        root.insert("Position", IVector3::new(0, 0, 0));

        Region::new_from_nbt(root).unwrap();
    }
}
