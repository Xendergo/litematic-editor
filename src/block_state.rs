use std::collections::HashMap;

use core::hash::Hash;
use quartz_nbt::{NbtCompound, NbtTag};

use crate::BlockStateParseError;

#[derive(Debug, PartialEq, Eq, Clone)]
pub struct BlockState {
    block: String,
    pub properties: HashMap<String, String>,
}

impl BlockState {
    pub fn new(block: &str, properties: Option<HashMap<String, String>>) -> BlockState {
        BlockState {
            block: BlockState::prefix_block_name(block),
            properties: match properties {
                Some(v) => v,
                None => HashMap::new(),
            },
        }
    }

    pub fn get_block(&self) -> &String {
        &self.block
    }

    pub fn set_block(&mut self, block: &str) {
        self.block = BlockState::prefix_block_name(block)
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

    fn prefix_block_name(name: &str) -> String {
        if !name.contains(":") {
            return "minecraft:".to_string() + &name;
        }

        name.to_string().to_lowercase()
    }
}

impl Into<NbtTag> for &BlockState {
    fn into(self) -> NbtTag {
        let mut compound = NbtCompound::new();
        let mut properties = NbtCompound::new();

        for (name, value) in self.properties.iter() {
            properties.insert(name, value);
        }

        compound.insert("Name", self.block.clone());
        compound.insert("Properties", properties);

        NbtTag::Compound(compound)
    }
}

impl Hash for BlockState {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.block.hash(state);

        for (key, value) in self.properties.iter() {
            key.hash(state);
            value.hash(state);
        }
    }
}
