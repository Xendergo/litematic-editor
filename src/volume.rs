use crate::IVector3;

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
