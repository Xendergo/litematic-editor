use quartz_nbt::{NbtCompound, NbtReprError};

#[derive(Hash, PartialEq, Eq, Debug, Clone, Copy)]
pub struct IVector3 {
    pub x: i32,
    pub y: i32,
    pub z: i32,
}

impl IVector3 {
    pub fn new(x: i32, y: i32, z: i32) -> IVector3 {
        return IVector3 { x: x, y: y, z: z };
    }

    pub(crate) fn from_nbt(nbt: &NbtCompound, name: &str) -> Result<IVector3, NbtReprError> {
        let vec_nbt = nbt.get::<_, &NbtCompound>(name)?;

        Ok(IVector3::new(
            vec_nbt.get::<_, i32>("x")?,
            vec_nbt.get::<_, i32>("y")?,
            vec_nbt.get::<_, i32>("z")?,
        ))
    }

    pub fn volume(&self) -> i32 {
        (self.x * self.y * self.z).abs()
    }
}
