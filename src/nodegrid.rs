use std::cmp;

use anyhow::{Result, anyhow};
use ratatui::{
    buffer::Buffer,
    layout::Rect,
    style::Style,
    widgets::{Block, Widget},
};
use serde::{Deserialize, Serialize};

use super::{NODE_H_SPACING, NODE_HEIGHT, NODE_V_SPACING, NODE_WIDTH};

use crate::{
    location::Location,
    node::{Node, NodeWidget},
};

#[derive(Debug, Default, Clone, Serialize, Deserialize)]
pub struct NodeGrid {
    pub(crate) nodes: Vec<Node>,
    pub(crate) floating_nodes: Vec<Node>,
}

#[derive(Debug, Default, Clone)]
pub struct NodeGridDisplay<'a> {
    pub(crate) grid: NodeGrid,
    pub(crate) block: Option<Block<'a>>,
}

impl NodeGrid {
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

    pub(crate) fn new_node(&mut self, name: String) -> Result<()> {
        let name = name.trim().to_string();
        if name.is_empty() {
            return Err(anyhow!("Name not unique."));
        }
        if self.nodes.iter().any(|n| n.name == name) {
            return Err(anyhow!("Name not unique."));
        }
        self.floating_nodes.push(Node {
            name,
            id: self.next_id(),
            ..Default::default()
        });
        anyhow::Ok(())
    }

    fn next_id(&self) -> usize {
        self.nodes.iter().max_by_key(|&n| n.id).unwrap().id + 1
    }

    pub(crate) fn move_node(&mut self, x: i8, y: i8) {
        for node in self.floating_nodes.iter_mut() {
            let mut location = node.location;
            location.horizontal = cmp::max(location.horizontal as i32 - x as i32, 0) as u16;
            location.vertical = cmp::max(location.vertical as i32 - y as i32, 0) as u16;
            node.location = location;
        }
    }

    /// Try to place `floating_nodes` back into the `nodes`.
    pub(crate) fn commit(&mut self) -> Result<()> {
        let overlap = self
            .nodes
            .iter()
            .any(|n| self.floating_nodes.iter().any(|m| m.location == n.location));
        match overlap {
            true => Err(anyhow!("Overlap in nodes")),
            false => {
                for node in self.floating_nodes.drain(0..) {
                    self.nodes.push(node.clone());
                }
                Ok(())
            }
        }
    }
}

impl Widget for NodeGrid {
    fn render(self, area: Rect, buf: &mut Buffer)
    where
        Self: Sized,
    {
        for node in self.nodes.iter() {
            let style = Style::default().fg(ratatui::style::Color::Green);
            let (x, y) = self.place(node);
            let node_widget = NodeWidget::from(node, style);
            let area = Rect::new(x, y, NODE_WIDTH, NODE_HEIGHT);
            node_widget.render(area, buf);
        }
        for node in self.floating_nodes.iter() {
            let style = Style::default().fg(ratatui::style::Color::Cyan);
            let (x, y) = self.place(node);
            let node_widget = NodeWidget::from(node, style);
            let area = Rect::new(x, y, NODE_WIDTH, NODE_HEIGHT);
            node_widget.render(area, buf);
        }
    }
}

impl<'a> NodeGridDisplay<'a> {
    pub fn new(grid: NodeGrid) -> Self {
        Self { grid, block: None }
    }

    /// Surrounds the `NodeGrid` with a `Block`.
    pub fn block(mut self, block: Block<'a>) -> Self {
        self.block = Some(block);
        self
    }
}

impl Widget for NodeGridDisplay<'_> {
    fn render(self, area: Rect, buf: &mut Buffer)
    where
        Self: Sized,
    {
        self.grid.render(area, buf);
        self.block.render(area, buf);
    }
}
