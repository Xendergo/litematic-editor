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
        let properties_nbt = match data.get::<_, &NbtTag>("Properties").ok() {
            Some(v) => v.clone(),
            None => NbtTag::Compound(NbtCompound::new()),
        };

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
            block: BlockState::prefix_block_name(data.get::<_, &str>("Name")?),
            properties: parsed_properties,
        })
    }

    fn prefix_block_name(name: &str) -> String {
        if !name.contains(":") {
            return ("minecraft:".to_string() + &name).to_lowercase();
        }

        name.to_lowercase()
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

        if properties.len() > 0 {
            compound.insert("Properties", properties);
        }

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

#[cfg(test)]
mod tests {
    use std::collections::HashMap;

    use quartz_nbt::NbtCompound;

    use crate::BlockState;

    #[test]
    fn test_prefix_block_name() {
        assert_eq!(BlockState::prefix_block_name("air"), "minecraft:air");
        assert_eq!(BlockState::prefix_block_name("Air"), "minecraft:air");
        assert_eq!(BlockState::prefix_block_name("CoOlMoD:aIr"), "coolmod:air");
    }

    #[test]
    fn test_new_from_nbt() {
        let mut compound = NbtCompound::new();

        compound.insert("Name", "bruh");

        assert_eq!(
            BlockState::new_from_nbt(&compound).unwrap(),
            BlockState::new("bruh", None)
        );

        let mut compound = NbtCompound::new();

        compound.insert("Name", "observer");

        let mut properties = NbtCompound::new();

        properties.insert("facing", "west");
        properties.insert("powered", "false");

        compound.insert("Properties", properties);

        let mut properties_map = HashMap::new();

        properties_map.insert("facing".to_string(), "west".to_string());
        properties_map.insert("powered".to_string(), "false".to_string());

        assert_eq!(
            BlockState::new_from_nbt(&compound).unwrap(),
            BlockState::new("observer", Some(properties_map))
        )
    }
}
