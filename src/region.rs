use std::collections::HashMap;

use quartz_nbt::{NbtCompound, NbtTag};

use crate::{BlockState, IVector3, RegionParseError};

pub struct Region {
    pub position: IVector3,
    block_state_palette: Vec<BlockState>,
    blocks: HashMap<IVector3, usize>,
}

impl Region {
    pub(crate) fn new_from_nbt(data: NbtCompound) -> Result<Region, RegionParseError> {
        let palette =
            if let NbtTag::List(palette_list) = data.get::<_, &NbtTag>("BlockStatePalette")? {
                palette_list
            } else {
                return Err(RegionParseError::WrongTag("BlockStatePalette".to_string()));
            };

        let mut parsed_palette = Vec::new();

        for state_unknown in palette {
            if let NbtTag::Compound(state) = state_unknown {
                parsed_palette.push(BlockState::new_from_nbt(state)?)
            }
        }

        let size_nbt = data.get::<_, &NbtCompound>("Size")?;

        let size = IVector3::new(
            size_nbt.get::<_, i32>("x")?,
            size_nbt.get::<_, i32>("y")?,
            size_nbt.get::<_, i32>("z")?,
        );

        // https://github.com/maruohon/litematica/issues/53#issuecomment-520281566
        let blocks_long_array =
            if let NbtTag::LongArray(array) = data.get::<_, &NbtTag>("BlockStates")? {
                array
            } else {
                return Err(RegionParseError::WrongTag("BlockStates".to_string()));
            };

        // Calculate number of bits used per block
        let bits = (usize::BITS - (parsed_palette.len() - 1).leading_zeros()).max(2) as u64;

        let blocks = blocks_long_array.len() as u64 * 64 / bits;

        let mut unpacked = HashMap::new();

        for block in 0..blocks {
            let pos = block * bits;

            let pos_in_long = pos % 64;

            let index = (pos - pos_in_long) as usize;

            let bitmap = ((1_i64 << bits) - 1_i64).rotate_left(pos_in_long as u32);

            let mut value = blocks_long_array[index] & bitmap >> pos_in_long;

            if index < blocks_long_array.len() - 1 {
                value |= blocks_long_array[index + 1] & bitmap << (64 - pos_in_long);
            }

            let y = block / (size.x * size.z) as u64;
            let z = (block % (size.x * size.z) as u64) / size.x as u64;
            let x = (block % (size.x * size.z) as u64) % size.x as u64;

            unpacked.insert(IVector3::new(x as i32, y as i32, z as i32), value as usize);
        }

        let position_nbt = data.get::<_, &NbtCompound>("Position")?;

        let position = IVector3::new(
            position_nbt.get::<_, i32>("x")?,
            position_nbt.get::<_, i32>("y")?,
            position_nbt.get::<_, i32>("z")?,
        );

        Ok(Region {
            position: position,
            block_state_palette: parsed_palette,
            blocks: unpacked,
        })
    }
}
