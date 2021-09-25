use quartz_nbt::{
    io::{self, Flavor, NbtIoError},
    NbtCompound, NbtTag,
};
use std::{collections::HashMap, io::Read};

use crate::{vector::Volume, LitematicParseError, Region};

pub struct Schematic {
    pub author: String,
    pub description: String,
    pub name: String,
    pub region_count: i32,
    pub time_created: i64,
    pub time_modified: i64,
    pub regions: HashMap<String, Region>,
    pub data_version: i32,
}

impl Schematic {
    pub fn from_buffer(data: &mut impl Read) -> Result<Schematic, LitematicParseError> {
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
            data_version: parsed_data.get::<_, i32>("MinecraftDataVersion")?,
        })
    }

    pub fn to_buffer(&self) -> Result<Vec<u8>, NbtIoError> {
        let mut out = NbtCompound::new();

        let mut metadata = NbtCompound::new();

        metadata.insert("Author", self.author.clone());
        metadata.insert("Description", self.description.clone());
        metadata.insert("RegionCount", self.regions.len() as i32);
        metadata.insert("TimeCreated", self.time_created);
        metadata.insert("TimeModified", self.time_modified);
        metadata.insert(
            "TotalBlocks",
            self.regions.values().fold(0, |a, v| a + v.total_blocks()),
        );

        let mut regions = NbtCompound::new();

        let mut total_volume: Option<Volume> = None;

        for (name, region) in self.regions.iter() {
            let (encoded, volume) = region.to_nbt();

            total_volume = Some(match total_volume {
                Some(v) => v.expand_to_fit_volume(volume),
                None => volume,
            });

            regions.insert(name, encoded);
        }

        let total_volume = total_volume.unwrap_or_default();

        metadata.insert("EnclosingSize", total_volume.size());
        metadata.insert("TotalVolume", total_volume.volume());

        out.insert("Metadata", metadata);
        out.insert("MinecraftDataVersion", self.data_version);
        out.insert("Version", 5);
        out.insert("Regions", regions);

        let mut out_buffer = Vec::new();

        io::write_nbt(&mut out_buffer, None, &out, Flavor::GzCompressed)?;

        Ok(out_buffer)
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
