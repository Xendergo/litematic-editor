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
        let mut pos1 = self.pos1.into_slice();
        let mut pos2 = self.pos2.into_slice();
        let vector = vector.into_slice();

        for i in 0..3 {
            if pos1[i] < pos2[i] {
                if vector[i] + 1 > pos2[i] {
                    pos2[i] = vector[i] + 1
                }

                if vector[i] < pos1[i] {
                    pos1[i] = vector[i]
                }
            } else if pos1[i] > pos2[i] {
                if vector[i] + 1 > pos1[i] {
                    pos1[i] = vector[i] + 1
                }

                if vector[i] < pos2[i] {
                    pos2[i] = vector[i]
                }
            } else {
                if vector[i] >= pos1[i] {
                    pos2[i] = vector[i] + 1;
                } else {
                    pos2[i] = vector[i]
                }
            }
        }

        Volume {
            pos1: IVector3::from_slice(pos1),
            pos2: IVector3::from_slice(pos2),
        }
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

    pub fn change_size(self, new_size: IVector3) -> Volume {
        Volume {
            pos1: self.pos1,
            pos2: self.pos1 + new_size,
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
mod tests {
    use crate::{IVector3, Volume};

    #[test]
    fn test_new() {
        assert_eq!(
            Volume::new(IVector3::new(0, 0, 0), IVector3::new(1, 1, 1)),
            Volume {
                pos1: IVector3::new(0, 0, 0),
                pos2: IVector3::new(1, 1, 1)
            }
        );

        assert_eq!(
            Volume::new(IVector3::new(1, 1, 1), IVector3::new(1, 1, 1)),
            Volume {
                pos1: IVector3::new(1, 1, 1),
                pos2: IVector3::new(2, 2, 2)
            }
        );

        assert_eq!(
            Volume::new(IVector3::new(1, 1, 1), IVector3::new(-1, -1, -1)),
            Volume {
                pos1: IVector3::new(1, 1, 1),
                pos2: IVector3::new(0, 0, 0)
            }
        );
    }

    #[test]
    fn test_expand_to_fit() {
        assert_eq!(
            Volume::default().expand_to_fit(IVector3::new(1, 1, 1)),
            Volume::new(IVector3::new(0, 0, 0), IVector3::new(2, 2, 2))
        );

        assert_eq!(
            Volume::default().expand_to_fit(IVector3::new(-1, -1, -1)),
            Volume::new(IVector3::new(0, 0, 0), IVector3::new(-1, -1, -1))
        );

        assert_eq!(
            Volume::new(IVector3::new(1, 1, 1), IVector3::new(2, 2, 2))
                .expand_to_fit(IVector3::new(0, 0, 0)),
            Volume::new(IVector3::new(0, 0, 0), IVector3::new(3, 3, 3))
        );

        assert_eq!(
            Volume::new(IVector3::new(1, 1, 1), IVector3::new(2, -2, 2))
                .expand_to_fit(IVector3::new(0, 0, 0)),
            Volume::new(IVector3::new(0, 1, 0), IVector3::new(3, -2, 3))
        );
    }
}
