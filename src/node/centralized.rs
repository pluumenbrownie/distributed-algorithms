use crate::nodegrid::algorithm_traits::*;

use super::Node;

impl Centralized for Node {
    fn initiator(&self) -> bool {
        self.name == *"p0"
    }
}
