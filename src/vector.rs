use quartz_nbt::{NbtCompound, NbtReprError};

#[derive(Hash, PartialEq, Eq, Debug, Clone, Copy)]
pub struct IVector3 {
    pub x: i32,
    pub y: i32,
    pub z: i32,
}

impl IVector3 {
    pub const fn new(x: i32, y: i32, z: i32) -> IVector3 {
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

    pub(crate) fn to_nbt(&self) -> NbtCompound {
        let mut compound = NbtCompound::new();

        compound.insert("x", self.x);
        compound.insert("y", self.y);
        compound.insert("z", self.z);

        compound
    }

    pub const fn volume(&self) -> i32 {
        (self.x * self.y * self.z).abs()
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_volume() {
        assert_eq!(IVector3::new(2, 3, 4).volume(), 24);
        assert_eq!(IVector3::new(0, 3, 4).volume(), 0);
        assert_eq!(IVector3::new(-2, 3, 4).volume(), 24);
    }

    #[test]
    fn test_from_nbt() {
        let mut root = NbtCompound::new();

        let mut vec_nbt = NbtCompound::new();

        vec_nbt.insert("x", 2);
        vec_nbt.insert("y", 3);
        vec_nbt.insert("z", 4);

        root.insert("size", vec_nbt);

        assert_eq!(
            IVector3::from_nbt(&root, "size").unwrap(),
            IVector3::new(2, 3, 4)
        );
    }

    #[test]
    fn test_to_nbt() {
        let nbt = IVector3::new(2, 3, 4).to_nbt();

        assert_eq!(nbt.get::<_, i32>("x").unwrap(), 2);
        assert_eq!(nbt.get::<_, i32>("y").unwrap(), 3);
        assert_eq!(nbt.get::<_, i32>("z").unwrap(), 4);
    }
}
