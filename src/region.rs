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
                Region::calculate_bits(&parsed_palette),
                IVector3::from_nbt(&data, "Size")?,
            ),
            block_state_palette: parsed_palette,
        })
    }

    fn calculate_bits(parsed_palette: &Vec<BlockState>) -> u64 {
        (usize::BITS - (parsed_palette.len() - 1).leading_zeros()).max(2) as u64
    }

    fn parse_palette(palette: &NbtList) -> Result<Vec<BlockState>, BlockStateParseError> {
        let mut parsed_palette = Vec::new();

        for state_unknown in palette {
            if let NbtTag::Compound(state) = state_unknown {
                parsed_palette.push(BlockState::new_from_nbt(&state)?)
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
                Region::index_to_coords(region_size, block),
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

        let index = (pos - pos_in_long) as usize;

        let bitmap = ((1_i64 << bits_per_position) - 1_i64).rotate_left(pos_in_long as u32);

        let mut value = array[index] & bitmap >> pos_in_long;

        if index < array.len() - 1 {
            value |= array[index + 1] & bitmap << (64 - pos_in_long);
        }

        value as usize
    }

    fn index_to_coords(size: IVector3, index: u64) -> IVector3 {
        let y = index / (size.x * size.z) as u64;
        let z = (index % (size.x * size.z) as u64) / size.x as u64;
        let x = (index % (size.x * size.z) as u64) % size.x as u64;

        IVector3::new(x as i32, y as i32, z as i32)
    }
}
