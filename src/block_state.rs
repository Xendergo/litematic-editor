use std::collections::HashMap;

use quartz_nbt::{NbtCompound, NbtTag};

use crate::BlockStateParseError;

#[derive(Debug)]
pub struct BlockState {
    pub block: String,
    pub properties: HashMap<String, String>,
}

impl BlockState {
    pub fn new(block: &str, properties: Option<HashMap<String, String>>) -> BlockState {
        BlockState {
            block: block.to_string(),
            properties: match properties {
                Some(v) => v,
                None => HashMap::new(),
            },
        }
    }

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
            block: data.get::<_, &String>("Name")?.clone(),
            properties: parsed_properties,
        })
    }

    pub(crate) fn to_nbt(&self) -> NbtCompound {
        let mut compound = NbtCompound::new();
        let mut properties = NbtCompound::new();

        for (name, value) in self.properties.iter() {
            properties.insert(name, value);
        }

        compound.insert("Name", self.block.clone());
        compound.insert("Properties", properties);

        compound
    }
}
