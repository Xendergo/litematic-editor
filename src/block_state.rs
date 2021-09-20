use std::collections::HashMap;

use quartz_nbt::{NbtCompound, NbtTag};

use crate::BlockStateParseError;

pub struct BlockState {
    pub name: String,
    pub properties: HashMap<String, String>,
}

impl BlockState {
    pub(crate) fn new_from_nbt(data: &NbtCompound) -> Result<BlockState, BlockStateParseError> {
        let properties_nbt = data.get::<_, &NbtTag>("Properties")?.clone();
        let properties_compound = if let NbtTag::Compound(properties_compound) = properties_nbt {
            properties_compound
        } else {
            return Err(BlockStateParseError::WrongTag("Properties".to_string()));
        };

        let mut parsed_properties = HashMap::new();

        for (name, tag) in properties_compound.inner() {
            if let NbtTag::String(value) = tag {
                parsed_properties.insert(name.clone(), value.clone());
            }
        }

        Ok(BlockState {
            name: data.get::<_, &String>("Name")?.clone(),
            properties: parsed_properties,
        })
    }
}
