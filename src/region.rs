use std::collections::HashMap;

use quartz_nbt::{NbtCompound, NbtList, NbtTag};

use crate::{vector::Volume, BlockState, BlockStateParseError, IVector3, RegionParseError};

pub struct Region {
    pub position: IVector3,
    block_state_palette: Vec<BlockState>,
    blocks: HashMap<IVector3, usize>,
    entities: Option<NbtList>,
    pending_block_ticks: Option<NbtList>,
    pending_fluid_ticks: Option<NbtList>,
    tile_entities: Option<NbtList>,
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

    fn calculate_bits(parsed_palette_length: usize) -> u64 {
        (usize::BITS - (parsed_palette_length - 1).leading_zeros()).max(2) as u64
    }

    fn calculate_amt_of_longs(region_volume: i32, bits: u64) -> i32 {
        ((region_volume * bits as i32) + (region_volume * bits as i32 % 64)) / 64
    }

    fn parse_palette(palette: &NbtList) -> Result<Vec<BlockState>, BlockStateParseError> {
        let mut parsed_palette = Vec::new();

        for state_unknown in palette {
            if let NbtTag::Compound(state) = state_unknown {
                parsed_palette.push(BlockState::new_from_nbt(&state)?)
            } else {
                return Err(BlockStateParseError::WrongTag(
                    "BlockStatePalette".to_string(),
                ));
            }
        }

        Ok(parsed_palette)
    }

    fn unpack_packed_array(
        array: &[i64],
        bits_per_position: u64,
        region_size: IVector3,
    ) -> HashMap<IVector3, usize> {
        let blocks = array.len() as u64 * 64 / bits_per_position;

        let mut unpacked = HashMap::new();

        for block in 0..blocks {
            let coords = match Region::index_to_coords(region_size, block) {
                Some(v) => v,
                None => unreachable!(),
            };

            unpacked.insert(
                coords,
                Region::get_index_out_of_packed_array(array, block, bits_per_position),
            );
        }

        unpacked
    }

    fn get_index_out_of_packed_array(
        array: &[i64],
        position_in_array: u64,
        bits_per_position: u64,
    ) -> usize {
        let pos = position_in_array * bits_per_position;

        let pos_in_long = pos % 64;
        let index = pos as usize / 64;

        let bitmap = ((1_i64 << bits_per_position) - 1_i64).rotate_left(pos_in_long as u32);

        let mut value = (array[index] & bitmap) >> pos_in_long;

        if index < array.len() - 1 {
            let amt_to_shift = 64 - pos_in_long as u32;
            value |= (array[index + 1] & bitmap)
                .checked_shl(amt_to_shift)
                .unwrap_or(0);
        }

        value as usize
    }

    fn set_index_in_packed_array(
        array: &mut [i64],
        value: i64,
        position_in_array: u64,
        bits_per_position: u64,
    ) {
        let pos = position_in_array * bits_per_position;

        let pos_in_long = pos % 64;
        let index = pos as usize / 64;

        let bitmap_1 = ((1_i64 << bits_per_position) - 1_i64) << pos_in_long;
        let rotated_value = value.rotate_left(pos_in_long as u32);

        array[index] &= !bitmap_1;
        array[index] |= rotated_value & bitmap_1;

        if index < array.len() - 1 {
            let amt_to_shift = 64 - pos_in_long as u32;
            let bitmap_2 = ((1_i64 << bits_per_position) - 1_i64)
                .checked_shr(amt_to_shift)
                .unwrap_or(0);
            array[index] |= (array[index + 1] & bitmap_2)
                .checked_shl(amt_to_shift)
                .unwrap_or(0);
        }
    }

    fn index_to_coords(size: IVector3, index: u64) -> Option<IVector3> {
        if size.volume() as u64 <= index {
            return None;
        }

        let y = index / (size.x * size.z) as u64;
        let z = (index % (size.x * size.z) as u64) / size.x as u64;
        let x = (index % (size.x * size.z) as u64) % size.x as u64;

        Some(IVector3::new(x as i32, y as i32, z as i32))
    }

    fn coords_to_index(size: IVector3, pos: IVector3) -> Option<u64> {
        if !pos.fits_in_positive(IVector3::new(0, 0, 0)) || !pos.fits_in_negative(size) {
            return None;
        }

        Some((pos.y * size.x * size.y + pos.z * size.x + pos.x) as u64)
    }

    fn write_misc_data(&self, data: &mut NbtCompound) {
        if let Some(v) = &self.entities {
            data.insert("Entities", v.clone());
        }

        if let Some(v) = &self.pending_block_ticks {
            data.insert("PendingBlockTicks", v.clone());
        }

        if let Some(v) = &self.pending_fluid_ticks {
            data.insert("PendingFluidTicks", v.clone());
        }

        if let Some(v) = &self.tile_entities {
            data.insert("TileEntities", v.clone());
        }
    }

    fn generate_palette_nbt(&self) -> NbtList {
        let mut palette = NbtList::new();

        for item in self.block_state_palette.iter() {
            palette.push(item.to_nbt());
        }

        palette
    }

    fn generate_block_states_nbt(&self, region_volume: Volume) -> Vec<i64> {
        let bits = Region::calculate_bits(self.block_state_palette.len());

        let longs = Region::calculate_amt_of_longs(region_volume.volume(), bits);

        let mut block_states: Vec<i64> = Vec::with_capacity(longs as usize);

        for _ in 0..longs {
            block_states.push(0);
        }

        let size = region_volume.size();
        let region_pos = region_volume.origin();

        for (block_pos, value) in self.blocks.iter() {
            Region::set_index_in_packed_array(
                &mut block_states,
                *value as i64,
                match Region::coords_to_index(size, *block_pos - region_pos) {
                    Some(v) => v,
                    None => unreachable!(),
                },
                bits,
            )
        }

        block_states
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
    fn test_bits_required() {
        assert_eq!(Region::calculate_bits(0b1), 2);
        assert_eq!(Region::calculate_bits(0b10), 2);
        assert_eq!(Region::calculate_bits(0b11), 2);
        assert_eq!(Region::calculate_bits(0b1111), 4);
        assert_eq!(Region::calculate_bits(0b10000), 4);
        assert_eq!(Region::calculate_bits(0b110110), 6);
        assert_eq!(Region::calculate_bits(0b10101010000), 11);
    }

    #[test]
    fn test_parse_palette() {
        let mut list = NbtList::new();

        list.push(BlockState::new("bruh", None).to_nbt());
        list.push(BlockState::new("yeet", None).to_nbt());
        list.push(BlockState::new("poggers?", None).to_nbt());

        let parsed = Region::parse_palette(&list);

        if let Ok(parsed_ok) = parsed {
            assert_eq!(parsed_ok.len(), 3);
            assert_eq!(parsed_ok[0].block, "bruh");
            assert_eq!(parsed_ok[1].block, "yeet");
            assert_eq!(parsed_ok[2].block, "poggers?");
        } else {
            panic!("parse_palette errored when it shouldn't have")
        }

        let mut list2 = NbtList::new();

        list2.push(1);
        list2.push(2);
        list2.push(3);

        let parsed2 = Region::parse_palette(&list2);

        if let Ok(_) = parsed2 {
            panic!("parse_palette succeeded when it shouldn't have")
        } else if let Err(err) = parsed2 {
            if let BlockStateParseError::WrongTag(_) = err {
                // correct
            } else {
                panic!("parse_palette failed for the wrong reason")
            }
        }
    }

    #[test]
    fn test_index_to_coords() {
        assert_eq!(
            Region::index_to_coords(IVector3::new(4, 6, 5), 53),
            Some(IVector3::new(1, 2, 3))
        );

        assert_eq!(Region::index_to_coords(IVector3::new(0, 0, 0), 0), None);
        assert_eq!(Region::index_to_coords(IVector3::new(0, 0, 0), 1), None);

        assert_eq!(
            Region::index_to_coords(IVector3::new(4, 6, 5), 54),
            Some(IVector3::new(2, 2, 3))
        );

        assert_eq!(
            Region::index_to_coords(IVector3::new(2, 3, 3), 1),
            Some(IVector3::new(1, 0, 0))
        );

        assert_eq!(
            Region::index_to_coords(IVector3::new(2, 3, 3), 9),
            Some(IVector3::new(1, 1, 1))
        );
    }

    #[test]
    fn test_get_index_out_of_packed_array() {
        let array: &[i64] = &[0x0123456789abcdef, 0x1032547698badcfe];
        let bits = 7;

        assert_eq!(Region::get_index_out_of_packed_array(array, 0, bits), 111);
        assert_eq!(Region::get_index_out_of_packed_array(array, 1, bits), 27);
        assert_eq!(Region::get_index_out_of_packed_array(array, 10, bits), 115);
        assert_eq!(Region::get_index_out_of_packed_array(array, 9, bits), 124);
    }

    #[test]
    fn test_unpack_packed_array() {
        let array: &[i64] = &[0x0123456789abcdef, 0x1032547698badcfe];

        let unpacked = Region::unpack_packed_array(array, 7, IVector3::new(2, 3, 3));

        println!("{:?}", unpacked);

        assert_eq!(unpacked.get(&IVector3::new(0, 0, 0)), Some(&111));
        assert_eq!(unpacked.get(&IVector3::new(1, 0, 0)), Some(&27));
        assert_eq!(unpacked.get(&IVector3::new(0, 1, 2)), Some(&115));
        assert_eq!(unpacked.get(&IVector3::new(1, 1, 1)), Some(&124));
    }

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
