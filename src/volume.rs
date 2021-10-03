use crate::{IVector3, Region, Vector3};

/// A struct that represents a box in 3d
#[derive(Hash, PartialEq, Eq, Debug, Clone, Copy)]
pub struct Volume {
    pos1: Vector3<i32>,
    pos2: Vector3<i32>,
}

impl Volume {
    /// Create a volume with the given position and size
    ///
    /// Negative numbers are allowed for the size
    pub fn new(pos: Vector3<i32>, size: Vector3<i32>) -> Volume {
        Volume {
            pos1: pos,
            pos2: pos + size,
        }
    }

    /// Change the position and size of this volume so that it'll contain the volume given
    ///
    /// Favors changing the size over changing the position
    ///
    /// ```
    /// # use litematic_editor::{Volume, Vector3};
    /// let volume1 = Volume::new(Vector3::new(0, 0, 0), Vector3::new(1, 1, 2));
    /// let volume2 = Volume::new(Vector3::new(0, -1, 0), Vector3::new(1, 2, 1));
    ///
    /// assert_eq!(volume1.expand_to_fit_volume(volume2), Volume::new(Vector3::new(0, -1, 0), Vector3::new(1, 2, 2)));
    /// ```
    pub fn expand_to_fit_volume(self, volume: Volume) -> Volume {
        let volume_positive = volume.make_size_positive();

        self.expand_to_fit(volume_positive.pos1)
            .expand_to_fit(volume_positive.pos2 - Vector3::new(1, 1, 1))
    }

    /// Change the position and size of this volume so that it'll contain the vector given
    ///
    /// Considers a vector to be a 1x1x1 cube, not a point
    ///
    /// Favors changing the size over changing the position
    ///
    /// ```
    /// # use litematic_editor::{Volume, Vector3};
    /// let volume = Volume::new(Vector3::new(0, -1, 0), Vector3::new(1, 2, 1));
    /// let vec = Vector3::new(2, -2, 1);
    ///
    /// assert_eq!(volume.expand_to_fit(vec), Volume::new(Vector3::new(0, -2, 0), Vector3::new(3, 3, 2)))
    /// ```
    pub fn expand_to_fit(self, vector: Vector3<i32>) -> Volume {
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
            pos1: Vector3::from_slice(pos1),
            pos2: Vector3::from_slice(pos2),
        }
    }

    /// Get the origin point of this volume
    ///
    /// ```
    /// # use litematic_editor::{Volume, Vector3};
    /// assert_eq!(Volume::new(Vector3::new(1, 2, 3), Vector3::new(1, -2, 5)).origin(), Vector3::new(1, 2, 3));
    /// ```
    pub fn origin(self) -> Vector3<i32> {
        self.pos1
    }

    /// Get the size of this volume
    ///
    /// ```
    /// # use litematic_editor::{Volume, Vector3};
    /// assert_eq!(Volume::new(Vector3::new(2, 5, 4), Vector3::new(-2, 5, -9)).size(), Vector3::new(-2, 5, -9));
    /// ```
    pub fn size(self) -> Vector3<i32> {
        self.pos2 - self.pos1
    }

    /// Move this volume to a new position
    ///
    /// ```
    /// # use litematic_editor::{Volume, Vector3};
    /// let mut volume = Volume::new(Vector3::new(1, 2, 3), Vector3::new(4, 5, 6));
    ///
    /// volume = volume.move_to(Vector3::new(0, 0, 0));
    ///
    /// assert_eq!(volume.origin(), Vector3::new(0, 0, 0));
    /// assert_eq!(volume.size(), Vector3::new(4, 5, 6));
    /// ```
    pub fn move_to(self, pos: Vector3<i32>) -> Volume {
        Volume {
            pos1: pos,
            pos2: (self.pos2 - self.pos1) + pos,
        }
    }

    /// Change the size of this volume
    ///
    /// ```
    /// # use litematic_editor::{Volume, Vector3};
    /// let mut volume = Volume::new(Vector3::new(1, 2, 3), Vector3::new(4, 5, 6));
    ///
    /// volume = volume.change_size(Vector3::new(7, 8, 9));
    ///
    /// assert_eq!(volume.origin(), Vector3::new(1, 2, 3));
    /// assert_eq!(volume.size(), Vector3::new(7, 8, 9));
    /// ```
    pub fn change_size(self, new_size: Vector3<i32>) -> Volume {
        Volume {
            pos1: self.pos1,
            pos2: self.pos1 + new_size,
        }
    }

    /// Calculate the volume of this volume
    ///
    /// ```
    /// # use litematic_editor::{Volume, Vector3};
    /// assert_eq!(Volume::new(Vector3::new(2, 3, 4), Vector3::new(3, 4, 5)).volume(), 60);
    /// ```
    pub fn volume(self) -> i32 {
        (self.pos2 - self.pos1).volume()
    }

    /// Change the origin and size of the volume so it contains the same area but the size has only positive values
    ///
    /// ```
    /// # use litematic_editor::{Volume, Vector3};
    /// let volume = Volume::new(Vector3::new(2, 3, 4), Vector3::new(-2, 5, -7));
    ///
    /// assert_eq!(volume.make_size_positive(), Volume::new(Vector3::new(0, 3, -3), Vector3::new(2, 5, 7)))
    /// ```
    pub fn make_size_positive(self) -> Volume {
        let mut pos1 = self.pos1.into_slice();
        let mut pos2 = self.pos2.into_slice();

        for i in 0..3 {
            if pos1[i] > pos2[i] {
                let tmp = pos1[i];
                pos1[i] = pos2[i];
                pos2[i] = tmp;
            }
        }

        Volume {
            pos1: Vector3::from_slice(pos1),
            pos2: Vector3::from_slice(pos2),
        }
    }

    /// Get an iterator over every block in the volume, increasing the x, then the z, then the y coordinates
    ///
    /// ```
    /// # use litematic_editor::{Volume, Vector3};
    /// let mut iter = Volume::new(Vector3::new(1, 1, 1), Vector3::new(2, 2, 2)).iter();
    ///
    /// assert_eq!(iter.next(), Some(Vector3::new(1, 1, 1)));
    /// assert_eq!(iter.next(), Some(Vector3::new(2, 1, 1)));
    /// assert_eq!(iter.next(), Some(Vector3::new(1, 1, 2)));
    /// assert_eq!(iter.next(), Some(Vector3::new(2, 1, 2)));
    /// assert_eq!(iter.next(), Some(Vector3::new(1, 2, 1)));
    /// assert_eq!(iter.next(), Some(Vector3::new(2, 2, 1)));
    /// assert_eq!(iter.next(), Some(Vector3::new(1, 2, 2)));
    /// assert_eq!(iter.next(), Some(Vector3::new(2, 2, 2)));
    /// assert_eq!(iter.next(), None);
    /// ```
    pub fn iter(self) -> VolumeIterator {
        VolumeIterator {
            volume: self.make_size_positive(),
            current_pos: 0,
        }
    }
}

pub struct VolumeIterator {
    volume: Volume,
    current_pos: u64,
}

impl Iterator for VolumeIterator {
    type Item = IVector3;

    fn next(&mut self) -> Option<Self::Item> {
        let ret = Region::index_to_coords(self.volume.size(), self.current_pos)
            .map(|v| v + self.volume.pos1);

        self.current_pos += 1;

        ret
    }
}

impl Default for Volume {
    fn default() -> Self {
        Volume {
            pos1: Vector3::default(),
            pos2: Vector3::default(),
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::{Vector3, Volume};

    #[test]
    fn test_new() {
        assert_eq!(
            Volume::new(Vector3::new(0, 0, 0), Vector3::new(1, 1, 1)),
            Volume {
                pos1: Vector3::new(0, 0, 0),
                pos2: Vector3::new(1, 1, 1)
            }
        );

        assert_eq!(
            Volume::new(Vector3::new(1, 1, 1), Vector3::new(1, 1, 1)),
            Volume {
                pos1: Vector3::new(1, 1, 1),
                pos2: Vector3::new(2, 2, 2)
            }
        );

        assert_eq!(
            Volume::new(Vector3::new(1, 1, 1), Vector3::new(-1, -1, -1)),
            Volume {
                pos1: Vector3::new(1, 1, 1),
                pos2: Vector3::new(0, 0, 0)
            }
        );
    }

    #[test]
    fn test_expand_to_fit() {
        assert_eq!(
            Volume::default().expand_to_fit(Vector3::new(1, 1, 1)),
            Volume::new(Vector3::new(0, 0, 0), Vector3::new(2, 2, 2))
        );

        assert_eq!(
            Volume::default().expand_to_fit(Vector3::new(-1, -1, -1)),
            Volume::new(Vector3::new(0, 0, 0), Vector3::new(-1, -1, -1))
        );

        assert_eq!(
            Volume::new(Vector3::new(1, 1, 1), Vector3::new(2, 2, 2))
                .expand_to_fit(Vector3::new(0, 0, 0)),
            Volume::new(Vector3::new(0, 0, 0), Vector3::new(3, 3, 3))
        );

        assert_eq!(
            Volume::new(Vector3::new(1, 1, 1), Vector3::new(2, -2, 2))
                .expand_to_fit(Vector3::new(0, 0, 0)),
            Volume::new(Vector3::new(0, 1, 0), Vector3::new(3, -2, 3))
        );
    }

    #[test]
    fn test_make_size_positive() {
        assert_eq!(
            Volume::new(Vector3::new(1, 1, 1), Vector3::new(1, -1, 1)).make_size_positive(),
            Volume::new(Vector3::new(1, 0, 1), Vector3::new(1, 1, 1))
        );

        assert_eq!(
            Volume::new(Vector3::new(1, 1, 1), Vector3::new(1, 1, 1)).make_size_positive(),
            Volume::new(Vector3::new(1, 1, 1), Vector3::new(1, 1, 1))
        );
    }

    #[test]
    fn test_iter() {
        let mut iter = Volume::new(Vector3::new(1, 1, 1), Vector3::new(2, 2, 2)).iter();

        assert_eq!(iter.next(), Some(Vector3::new(1, 1, 1)));
        assert_eq!(iter.next(), Some(Vector3::new(2, 1, 1)));
        assert_eq!(iter.next(), Some(Vector3::new(1, 1, 2)));
        assert_eq!(iter.next(), Some(Vector3::new(2, 1, 2)));
        assert_eq!(iter.next(), Some(Vector3::new(1, 2, 1)));
        assert_eq!(iter.next(), Some(Vector3::new(2, 2, 1)));
        assert_eq!(iter.next(), Some(Vector3::new(1, 2, 2)));
        assert_eq!(iter.next(), Some(Vector3::new(2, 2, 2)));
        assert_eq!(iter.next(), None);
    }
}
