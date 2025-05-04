use std::cmp::Ordering;

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub struct Location {
    pub(crate) horizontal: u16,
    pub(crate) vertical: u16,
}

impl Location {
    pub fn new(horizontal: u16, vertical: u16) -> Self {
        Location {
            horizontal,
            vertical,
        }
    }
}

impl PartialOrd for Location {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        match self.horizontal.cmp(&other.horizontal) {
            Ordering::Greater => match self.vertical.cmp(&other.vertical) {
                Ordering::Less => None,
                _ => Some(Ordering::Greater),
            },
            Ordering::Less => match self.vertical.cmp(&other.vertical) {
                Ordering::Greater => None,
                _ => Some(Ordering::Less),
            },
            Ordering::Equal => Some(self.vertical.cmp(&other.vertical)),
        }
    }
}
