#[derive(Hash, PartialEq, Eq, Debug)]
pub struct IVector3 {
    pub x: i32,
    pub y: i32,
    pub z: i32,
}

impl IVector3 {
    pub fn new(x: i32, y: i32, z: i32) -> IVector3 {
        return IVector3 { x: x, y: y, z: z };
    }

    pub fn volume(&self) -> i32 {
        (self.x * self.y * self.z).abs()
    }
}
