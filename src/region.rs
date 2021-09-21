use std::collections::HashMap;

use quartz_nbt::{NbtCompound, NbtList, NbtTag};

use crate::{BlockState, BlockStateParseError, IVector3, RegionParseError};

pub struct Region {
    pub position: IVector3,
    block_state_palette: Vec<BlockState>,
    blocks: HashMap<IVector3, usize>,
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
        })
    }

    fn calculate_bits(parsed_palette_length: usize) -> u64 {
        (usize::BITS - (parsed_palette_length - 1).leading_zeros()).max(2) as u64
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
            unpacked.insert(
                match Region::index_to_coords(region_size, block) {
                    Some(v) => v,
                    None => unreachable!(),
                },
                Region::get_index_out_of_packed_array(array, block, bits_per_position),
            );
        }

        unpacked
    }

    const fn get_index_out_of_packed_array(
        array: &[i64],
        position_in_array: u64,
        bits_per_position: u64,
    ) -> usize {
        let pos = position_in_array * bits_per_position;

        let pos_in_long = pos % 64;

        let index = (pos - pos_in_long) as usize;

        let bitmap = ((1_i64 << bits_per_position) - 1_i64).rotate_left(pos_in_long as u32);

        let mut value = array[index] & bitmap >> pos_in_long;

        if index < array.len() - 1 {
            value |= array[index + 1] & bitmap << (64 - pos_in_long);
        }

        value as usize
    }

    const fn index_to_coords(size: IVector3, index: u64) -> Option<IVector3> {
        if size.volume() as u64 <= index {
            return None;
        }

        let y = index / (size.x * size.z) as u64;
        let z = (index % (size.x * size.z) as u64) / size.x as u64;
        let x = (index % (size.x * size.z) as u64) % size.x as u64;

        Some(IVector3::new(x as i32, y as i32, z as i32))
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

        let list2 = NbtList::new();

        list.push(1);
        list.push(2);
        list.push(3);

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

        assert_eq!(
            Region::index_to_coords(IVector3::new(0, 0, 0), 0),
            Some(IVector3::new(0, 0, 0))
        );

        assert_eq!(Region::index_to_coords(IVector3::new(0, 0, 0), 1), None);

        assert_eq!(
            Region::index_to_coords(
                IVector3::new(1976356, 27939740, 76356758),
                146682748097123140
            ),
            Some(IVector3::new(1286, 972, 19791))
        );
    }

    #[test]
    fn test_get_index_out_of_packed_array() {
        let array: &[i64] = &[0x0123456789abcdef, 0x1032547698badcfe];
        let bits = 7;

        assert_eq!(Region::get_index_out_of_packed_array(array, 0, bits), 35);
        assert_eq!(Region::get_index_out_of_packed_array(array, 1, bits), 10112);
        assert_eq!(Region::get_index_out_of_packed_array(array, 10, bits), 4672);
        assert_eq!(Region::get_index_out_of_packed_array(array, 9, bits), 68);
    }

    #[test]
    fn test_unpack_packed_array() {
        let array: &[i64] = &[0x0123456789abcdef, 0x1032547698badcfe];

        let unpacked = Region::unpack_packed_array(array, 7, IVector3::new(2, 3, 3));

        assert_eq!(unpacked.get(&IVector3::new(0, 0, 0)), Some(&35));
        assert_eq!(unpacked.get(&IVector3::new(1, 0, 0)), Some(&10112));
        assert_eq!(unpacked.get(&IVector3::new(1, 1, 1)), Some(&4672));
        assert_eq!(unpacked.get(&IVector3::new(1, 1, 0)), Some(&68));
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

        root.insert("Size", IVector3::new(4, 4, 4).to_nbt());

        Region::new_from_nbt(root).unwrap();
    }
}
