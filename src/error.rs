use quartz_nbt::{io::NbtIoError, NbtReprError, NbtStructureError};
use thiserror::Error;

/// The error returned when attempting to parse a buffer as a schematic fails
#[derive(Error, Debug)]
pub enum LitematicParseError {
    /// The data given is invalid NBT data
    #[error("The data given is invalid NBT data: {0}")]
    NBTParseError(#[from] NbtIoError),
    /// The schematic gives a version number that is too old or too recent, contains the version number given
    #[error("The litematic file uses version {0}, which is unsupported")]
    UnsupportedVersionNumber(i32),
    /// When a tag doesn't exist or is the wrong type
    #[error("{0}")]
    NonexistentTag(Box<NbtStructureError>),
    /// When a tag is the wrong type, contains the name of the incorrect tag
    #[error("The tag {0} is the wrong type")]
    WrongTag(String),
    /// When there's an issue parsing one of the regions
    #[error("There was an error parsing a region: {0}")]
    RegionParseError(#[from] RegionParseError),
    /// Any unknown error occured
    #[error("An unknown error occured")]
    Unknown,
}

/// The error returned when attempting to parse NBT data as a region fails
#[derive(Error, Debug)]
pub enum RegionParseError {
    /// When a tag doesn't exist or is the wrong type
    #[error("{0}")]
    NonexistentTag(Box<NbtStructureError>),
    /// When parsing one of the block states fails
    #[error("Error parsing one of the block states: {0}")]
    BlockStateParseError(#[from] BlockStateParseError),
    /// When a tag is the wrong type, contains the name of the incorrect tag
    #[error("The tag {0} is the wrong type")]
    WrongTag(String),
    /// When any unknown error occurs
    #[error("An unknown error occured")]
    Unknown,
}

/// The error returned when attepmting to parse NBT data as a block state fails
#[derive(Error, Debug)]
pub enum BlockStateParseError {
    /// When a tag doesn't exist or is the wrong type
    #[error("{0}")]
    NonexistentTag(Box<NbtStructureError>),
    /// When a tag is the wrong type, contains the name of the incorrect tag
    #[error("The tag {0} is the wrong type")]
    WrongTag(String),
    /// When any unknown error occurs
    #[error("An unknown error occured")]
    Unknown,
}

impl From<NbtReprError> for LitematicParseError {
    fn from(data: NbtReprError) -> LitematicParseError {
        match data {
            NbtReprError::Structure(err) => LitematicParseError::NonexistentTag(err),
            NbtReprError::Custom(_) => LitematicParseError::Unknown,
        }
    }
}

impl From<NbtReprError> for RegionParseError {
    fn from(data: NbtReprError) -> RegionParseError {
        match data {
            NbtReprError::Structure(err) => RegionParseError::NonexistentTag(err),
            NbtReprError::Custom(_) => RegionParseError::Unknown,
        }
    }
}

impl From<NbtReprError> for BlockStateParseError {
    fn from(data: NbtReprError) -> BlockStateParseError {
        match data {
            NbtReprError::Structure(err) => BlockStateParseError::NonexistentTag(err),
            NbtReprError::Custom(_) => BlockStateParseError::Unknown,
        }
    }
}
