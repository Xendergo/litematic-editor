use std::collections::HashMap;

use quartz_nbt::{NbtCompound, NbtList, NbtTag};

use crate::{vector::Volume, BlockState, IVector3, RegionParseError};

pub struct Region {
    pub volume: Volume,
    pub(super) blocks: HashMap<IVector3, BlockState>,
    pub(super) entities: Option<NbtList>,
    pub(super) pending_block_ticks: Option<NbtList>,
    pub(super) pending_fluid_ticks: Option<NbtList>,
    pub(super) tile_entities: Option<NbtList>,
}

// https://github.com/maruohon/litematica/issues/53#issuecomment-520281566
impl Region {
    pub fn volume(&self) -> Volume {
        self.blocks
            .keys()
            .fold(self.volume, |volume, value| volume.expand_to_fit(*value))
    }

    pub fn set_block(&mut self, pos: IVector3, block: BlockState) {
        if block != BlockState::new("air", None) {
            self.blocks.insert(pos, block);
        }
    }

    pub fn blocks(&self) -> &HashMap<IVector3, BlockState> {
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

        let size = IVector3::from_nbt(&data, "Size")?;

        Ok(Region {
            volume: Volume::new(IVector3::from_nbt(&data, "Position")?, size),
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

        let palette = self.generate_palette_nbt();

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
            self.generate_block_states_nbt(volume, &palette),
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

        root.insert("Size", IVector3::new(4, 4, 4));
        root.insert("Position", IVector3::new(0, 0, 0));

        Region::new_from_nbt(root).unwrap();
    }
}
