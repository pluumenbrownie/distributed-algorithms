use super::NODE_HEIGHT;

use super::NODE_V_SPACING;

use super::NODE_WIDTH;

use super::NODE_H_SPACING;

use crate::node::Node;

use super::NodeGrid;

use std::cmp::Ordering;

use super::Location;

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

impl NodeGrid<'_> {
    pub(crate) fn new(nodes: Vec<Location>) -> Self {
        let mut grid = NodeGrid::default();

        for (id, location) in nodes.into_iter().enumerate() {
            grid.nodes.push(Node {
                name: "Yo".to_string(),
                id,
                connections: vec![],
                location,
            });
        }

        grid
    }
    pub(crate) fn place(&self, node: &Node) -> (u16, u16) {
        (
            NODE_H_SPACING + node.location.horizontal * (NODE_H_SPACING + NODE_WIDTH),
            NODE_V_SPACING + node.location.vertical * (NODE_V_SPACING + NODE_HEIGHT),
        )
    }
}
