use quartz_nbt::{io::NbtIoError, NbtReprError, NbtStructureError};
use thiserror::Error;

#[derive(Error, Debug)]
pub enum LitematicParseError {
    #[error("The data given is invalid NBT data: {0}")]
    NBTParseError(#[from] NbtIoError),
    #[error("The litematic file uses version {0}, which is unsupported")]
    UnsupportedVersionNumber(i32),
    #[error("{0}")]
    NonexistentTag(Box<NbtStructureError>),
    #[error("There was an error parsing a region: {0}")]
    RegionParseError(#[from] RegionParseError),
    #[error("An unknown error occured")]
    Unknown,
}

#[derive(Error, Debug)]
pub enum RegionParseError {
    #[error("{0}")]
    NonexistentTag(Box<NbtStructureError>),
    #[error("Error parsing one of the block states: {0}")]
    BlockStateParseError(#[from] BlockStateParseError),
    #[error("The tag {0} is the wrong type")]
    WrongTag(String),
    #[error("An unknown error occured")]
    Unknown,
}

#[derive(Error, Debug)]
pub enum BlockStateParseError {
    #[error("{0}")]
    NonexistentTag(Box<NbtStructureError>),
    #[error("The tag {0} is the wrong type")]
    WrongTag(String),
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
