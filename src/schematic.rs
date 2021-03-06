use quartz_nbt::{
    io::{self, Flavor},
    NbtCompound, NbtTag,
};
use std::{collections::HashMap, io::Read};

use crate::{volume::Volume, LitematicParseError, Region};

/// A struct that stores the data in a schematic
pub struct Schematic {
    /// A schematic's author
    pub author: String,
    /// A schematic's description
    pub description: String,
    /// A schematic's name
    pub name: String,
    time_created: i64,
    /// Last time a schematic was modified, in milliseconds since 1970
    pub time_modified: i64,
    /// A hashmap of the schematic's regions
    pub regions: HashMap<String, Region>,
    data_version: i32,
}

impl Schematic {
    /// Create a new schematic
    pub fn new(
        name: Option<String>,
        author: Option<String>,
        description: Option<String>,
        time_created: Option<i64>,
    ) -> Schematic {
        Schematic {
            name: name.unwrap_or("".to_string()),
            author: author.unwrap_or("".to_string()),
            description: description.unwrap_or("".to_string()),
            time_created: time_created.unwrap_or(0),
            time_modified: time_created.unwrap_or(0),
            regions: HashMap::new(),
            data_version: 2730,
        }
    }

    /// Read a schematic from a buffer
    ///
    /// ```
    /// use litematic_editor::Schematic;
    /// use std::fs::File;
    ///
    /// # fn main() -> Result<(), std::io::Error> {
    /// let schematic = Schematic::from_buffer(&mut File::open("test/path/to/schematic.litematic")?);
    /// # Ok(())
    /// # }
    /// ```
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
            time_created: metadata.get::<_, i64>("TimeCreated")?,
            time_modified: metadata.get::<_, i64>("TimeModified")?,
            regions: Schematic::parse_regions(&parsed_data)?,
            data_version: parsed_data.get::<_, i32>("MinecraftDataVersion")?,
        })
    }

    /// Write a schematic's data to a u8 vector
    ///
    /// ```
    /// use litematic_editor::Schematic;
    /// use std::fs;
    ///
    /// # fn main() -> Result<(), std::io::Error> {
    /// let schematic = Schematic::new(Some("example schematic".to_string()), Some("a cool person".to_string()), None, None);
    ///
    /// let buffer = schematic.to_buffer();
    ///
    /// fs::write("test/path/to/new_schematic.litematic", buffer)?;
    /// # Ok(())
    /// # }
    /// ```
    pub fn to_buffer(&self) -> Vec<u8> {
        let mut out = NbtCompound::new();

        let mut metadata = NbtCompound::new();

        metadata.insert("Name", self.name.clone());
        metadata.insert("Author", self.author.clone());
        metadata.insert("Description", self.description.clone());
        metadata.insert("RegionCount", self.regions.len() as i32);
        metadata.insert("TimeCreated", self.time_created);
        metadata.insert("TimeModified", self.time_modified);
        metadata.insert(
            "TotalBlocks",
            self.regions
                .values()
                .fold(0, |a, v| a + v.blocks().len() as i32),
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

        io::write_nbt(&mut out_buffer, None, &out, Flavor::GzCompressed).unwrap();

        out_buffer
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
