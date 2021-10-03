use std::{
    convert::TryFrom,
    error::Error,
    ops::{Add, Sub},
};

use quartz_nbt::{NbtCompound, NbtReprError, NbtTag};

/// A Vector3 of i32 values
pub type IVector3 = Vector3<i32>;
/// A Vector3 of u32 values
pub type UVector3 = Vector3<u32>;
/// A Vector3 of f32 values
pub type FVector3 = Vector3<f32>;

/// A vector of three values, used to represent a position
#[derive(Hash, PartialEq, Eq, Debug, Clone, Copy)]
pub struct Vector3<T: Copy> {
    /// x position
    pub x: T,
    /// y position
    pub y: T,
    /// z position
    pub z: T,
}

impl<T: Copy> Vector3<T> {
    /// Create a new vector
    pub fn new(x: T, y: T, z: T) -> Vector3<T> {
        return Vector3 { x: x, y: y, z: z };
    }

    /// Convert a vector into a slice of three values
    pub fn into_slice(&self) -> [T; 3] {
        [self.x, self.y, self.z]
    }

    /// Convert a slice of three values into a vector
    pub fn from_slice([x, y, z]: [T; 3]) -> Vector3<T> {
        Vector3 { x: x, y: y, z: z }
    }
}

impl<'a, T> Vector3<T>
where
    T: Copy + TryFrom<&'a NbtTag>,
    <T as TryFrom<&'a NbtTag>>::Error: Error + Sync + Send + 'static,
{
    pub(crate) fn from_nbt(nbt: &'a NbtCompound, name: &str) -> Result<Vector3<T>, NbtReprError> {
        let vec_nbt = nbt.get::<_, &NbtCompound>(name)?;

        Ok(Vector3::new(
            vec_nbt.get::<'a, 'a, str, T>("x")?,
            vec_nbt.get::<'a, 'a, str, T>("y")?,
            vec_nbt.get::<'a, 'a, str, T>("z")?,
        ))
    }
}

impl Vector3<i32> {
    /// Check if all three coordinates of this vector are higher than the coordinates of the other vector
    ///
    /// ```
    /// # use litematic_editor::Vector3;
    /// let v1 = Vector3::new(1, 2, 3);
    ///
    /// assert_eq!(Vector3::new(2, 3, 4).fits_in_positive(v1), true);
    /// assert_eq!(Vector3::new(1, 2, 3).fits_in_positive(v1), true);
    /// assert_eq!(Vector3::new(1, 1, 3).fits_in_positive(v1), false);
    /// ```
    pub fn fits_in_positive(self, other: Vector3<i32>) -> bool {
        self.x >= other.x && self.y >= other.y && self.z >= other.z
    }

    /// Check if all three coordinates of this vector are lower than the coordinates of the other vector
    ///
    /// ```
    /// # use litematic_editor::Vector3;
    /// let v1 = Vector3::new(1, 2, 3);
    ///
    /// assert_eq!(Vector3::new(0, 1, 2).fits_in_negative(v1), true);
    /// assert_eq!(Vector3::new(1, 2, 3).fits_in_negative(v1), true);
    /// assert_eq!(Vector3::new(1, 2, 4).fits_in_negative(v1), false);
    /// ```
    pub fn fits_in_negative(self, other: Vector3<i32>) -> bool {
        self.x <= other.x && self.y <= other.y && self.z <= other.z
    }

    /// Get the volume of the volume between (0, 0, 0) and this vector
    ///
    /// ```
    /// # use litematic_editor::Vector3;
    /// assert_eq!(Vector3::new(0, 0, 0).volume(), 0);
    /// assert_eq!(Vector3::new(1, 2, 3).volume(), 6);
    /// assert_eq!(Vector3::new(-3, 2, 7).volume(), 42);
    /// ```
    pub fn volume(&self) -> i32 {
        (self.x * self.y * self.z).abs()
    }
}

impl<T: Copy + Add<Output = T>> Add for Vector3<T> {
    type Output = Vector3<T>;

    fn add(self, rhs: Self) -> Self::Output {
        let mut ret = self;

        ret.x = ret.x + rhs.x;
        ret.y = ret.y + rhs.y;
        ret.z = ret.z + rhs.z;

        ret
    }
}

impl<T: Copy + Sub<Output = T>> Sub for Vector3<T> {
    type Output = Vector3<T>;

    fn sub(self, rhs: Self) -> Self::Output {
        let mut ret = self;

        ret.x = ret.x - rhs.x;
        ret.y = ret.y - rhs.y;
        ret.z = ret.z - rhs.z;

        ret
    }
}

impl<T: Copy + Into<NbtTag>> Into<NbtTag> for Vector3<T> {
    fn into(self) -> NbtTag {
        let mut compound = NbtCompound::new();

        compound.insert("x", self.x);
        compound.insert("y", self.y);
        compound.insert("z", self.z);

        NbtTag::Compound(compound)
    }
}

impl Default for Vector3<i32> {
    fn default() -> Self {
        Vector3 { x: 0, y: 0, z: 0 }
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_volume() {
        assert_eq!(Vector3::new(2, 3, 4).volume(), 24);
        assert_eq!(Vector3::new(0, 3, 4).volume(), 0);
        assert_eq!(Vector3::new(-2, 3, 4).volume(), 24);
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
            Vector3::from_nbt(&root, "size").unwrap(),
            Vector3::new(2, 3, 4)
        );
    }

    #[test]
    fn test_to_nbt() {
        let nbt: NbtTag = Vector3::new(2, 3, 4).into();

        if let NbtTag::Compound(v) = nbt {
            assert_eq!(v.get::<_, i32>("x").unwrap(), 2);
            assert_eq!(v.get::<_, i32>("y").unwrap(), 3);
            assert_eq!(v.get::<_, i32>("z").unwrap(), 4);
        } else {
            panic!("`into` conversion didn't produce a compound");
        }
    }
}
