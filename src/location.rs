use std::cmp::Ordering;

use serde::{Deserialize, Serialize};

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, PartialOrd, Ord)]
pub struct Location {
    pub(crate) x: u16,
    pub(crate) y: u16,
}

impl Location {
    pub fn new(x: u16, y: u16) -> Self {
        Location { x, y }
    }

    /// Returns the topleft most point of the bounding box drawn by `self` and `other`.
    pub fn lowest(&self, other: &Location) -> Self {
        Location {
            x: self.x.min(other.x),
            y: self.y.min(other.y),
        }
    }
}

// impl PartialOrd for Location {
//     fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
//         match self.x.cmp(&other.x) {
//             Ordering::Greater => match self.y.cmp(&other.y) {
//                 Ordering::Less => None,
//                 _ => Some(Ordering::Greater),
//             },
//             Ordering::Less => match self.y.cmp(&other.y) {
//                 Ordering::Greater => None,
//                 _ => Some(Ordering::Less),
//             },
//             Ordering::Equal => Some(self.y.cmp(&other.y)),
//         }
//     }
// }
