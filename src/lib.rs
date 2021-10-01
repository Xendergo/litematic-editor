#![deny(rustdoc::broken_intra_doc_links)]
#![deny(rustdoc::private_intra_doc_links)]
#![warn(missing_docs)]

mod block_state;
mod error;
mod region;
mod schematic;
mod vector;
mod volume;

pub use block_state::BlockState;
pub use error::{BlockStateParseError, LitematicParseError, RegionParseError};
pub use region::Region;
pub use schematic::Schematic;
pub use vector::{FVector3, IVector3, UVector3, Vector3};
pub use volume::Volume;
