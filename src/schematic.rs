use quartz_nbt::{
    io::{self, Flavor},
    NbtCompound, NbtTag,
};
use std::{collections::HashMap, io::Read};

use crate::{LitematicParseError, Region};

pub struct Schematic {
    pub author: String,
    pub description: String,
    pub name: String,
    pub region_count: i32,
    pub time_created: i64,
    pub time_modified: i64,
    pub regions: HashMap<String, Region>,
}

impl Schematic {
    pub fn from_data(data: &mut impl Read) -> Result<Schematic, LitematicParseError> {
        let parsed_data = io::read_nbt(data, Flavor::GzCompressed)?.0;

        let version = parsed_data.get::<_, i32>("Version")?;

        if version != 5 {
            return Err(LitematicParseError::UnsupportedVersionNumber(version));
        }

        let metadata = parsed_data.get::<_, &NbtCompound>("Metadata")?;

        Ok(Schematic {
            author: metadata.get::<_, &String>("Author")?.clone(),
            description: metadata.get::<_, &String>("Description")?.clone(),
            name: metadata.get::<_, &String>("Name")?.clone(),
            region_count: metadata.get::<_, i32>("RegionCount")?,
            time_created: metadata.get::<_, i64>("TimeCreated")?,
            time_modified: metadata.get::<_, i64>("TimeModified")?,
            regions: Schematic::parse_regions(&parsed_data)?,
        })
    }

    fn parse_regions(data: &NbtCompound) -> Result<HashMap<String, Region>, LitematicParseError> {
        let regions_nbt =
            if let NbtTag::Compound(regions_nbt) = data.get::<_, &NbtTag>("Regions")?.clone() {
                regions_nbt
            } else {
                return Err(LitematicParseError::WrongTag("Regions".to_string()));
            };

        let regions = regions_nbt.into_inner();

        let mut regions_parsed = HashMap::new();

        for region_unknown in regions {
            if let NbtTag::Compound(region) = region_unknown.1 {
                regions_parsed.insert(region_unknown.0, Region::new_from_nbt(region)?);
            } else {
                return Err(LitematicParseError::WrongTag(region_unknown.0));
            }
        }

        Ok(regions_parsed)
    }
}
