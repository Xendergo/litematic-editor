use std::ops::{Add, Sub};

use quartz_nbt::{NbtCompound, NbtReprError, NbtTag};

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

    pub fn fits_in_positive(self, other: IVector3) -> bool {
        self.x > other.x && self.y > other.y && self.z > other.z
    }

    pub fn fits_in_negative(self, other: IVector3) -> bool {
        self.x < other.x && self.y < other.y && self.z < other.z
    }

    pub const fn volume(&self) -> i32 {
        (self.x * self.y * self.z).abs()
    }
}

impl Add for IVector3 {
    type Output = IVector3;

    fn add(self, rhs: Self) -> Self::Output {
        let mut ret = self;

        ret.x += rhs.x;
        ret.y += rhs.y;
        ret.z += rhs.z;

        ret
    }
}

impl Sub for IVector3 {
    type Output = IVector3;

    fn sub(self, rhs: Self) -> Self::Output {
        let mut ret = self;

        ret.x -= rhs.x;
        ret.y -= rhs.y;
        ret.z -= rhs.z;

        ret
    }
}

impl Into<NbtTag> for IVector3 {
    fn into(self) -> NbtTag {
        let mut compound = NbtCompound::new();

        compound.insert("x", self.x);
        compound.insert("y", self.y);
        compound.insert("z", self.z);

        NbtTag::Compound(compound)
    }
}

impl Default for IVector3 {
    fn default() -> Self {
        IVector3 { x: 0, y: 0, z: 0 }
    }
}

#[derive(Hash, PartialEq, Eq, Debug, Clone, Copy)]
pub struct Volume {
    pos1: IVector3,
    pos2: IVector3,
}

impl Volume {
    pub fn new(pos: IVector3, size: IVector3) -> Volume {
        Volume {
            pos1: pos,
            pos2: pos + size,
        }
    }

    pub fn expand_to_fit_volume(self, volume: Volume) -> Volume {
        self.expand_to_fit(volume.pos1).expand_to_fit(volume.pos2)
    }

    pub fn expand_to_fit(self, vector: IVector3) -> Volume {
        let mut ret = self;

        ret.pos1.x = if self.pos1.x > self.pos2.x {
            self.pos1.x.min(vector.x)
        } else {
            self.pos1.x.max(vector.x)
        };
        ret.pos1.y = if self.pos1.y > self.pos2.y {
            self.pos1.y.min(vector.y)
        } else {
            self.pos1.y.max(vector.y)
        };
        ret.pos1.z = if self.pos1.z > self.pos2.z {
            self.pos1.z.min(vector.z)
        } else {
            self.pos1.z.max(vector.z)
        };
        ret.pos2.x = if self.pos1.x < self.pos2.x {
            self.pos2.x.min(vector.x)
        } else {
            self.pos2.x.max(vector.x)
        };
        ret.pos2.y = if self.pos1.y < self.pos2.y {
            self.pos2.y.min(vector.y)
        } else {
            self.pos2.y.max(vector.y)
        };
        ret.pos2.z = if self.pos1.z < self.pos2.z {
            self.pos2.z.min(vector.z)
        } else {
            self.pos2.z.max(vector.z)
        };

        ret
    }

    pub fn origin(self) -> IVector3 {
        self.pos1
    }

    pub fn size(self) -> IVector3 {
        self.pos2 - self.pos1
    }

    pub fn move_to(self, pos: IVector3) -> Volume {
        Volume {
            pos1: pos,
            pos2: self.pos2 + pos,
        }
    }

    pub fn volume(self) -> i32 {
        (self.pos2 - self.pos1).volume()
    }
}

impl Default for Volume {
    fn default() -> Self {
        Volume {
            pos1: IVector3::default(),
            pos2: IVector3::default(),
        }
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
        let nbt: NbtTag = IVector3::new(2, 3, 4).into();

        if let NbtTag::Compound(v) = nbt {
            assert_eq!(v.get::<_, i32>("x").unwrap(), 2);
            assert_eq!(v.get::<_, i32>("y").unwrap(), 3);
            assert_eq!(v.get::<_, i32>("z").unwrap(), 4);
        } else {
            panic!("`into` conversion didn't produce a compound");
        }
    }
}
