use std::cmp;

use anyhow::{Ok, Result, anyhow};
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
    node::{self, Connection, Node, NodeWidget},
};

#[derive(Debug, Default, Clone, Serialize, Deserialize)]
pub struct NodeGrid {
    pub(crate) nodes: Vec<Node>,

    #[serde(skip)]
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
                ..Default::default()
            });
        }

        grid
    }

    pub(crate) fn place(&self, node: &Node) -> (u16, u16) {
        (
            NODE_H_SPACING + node.location.x * (NODE_H_SPACING + NODE_WIDTH),
            NODE_V_SPACING + node.location.y * (NODE_V_SPACING + NODE_HEIGHT),
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
        self.nodes.iter().max_by_key(|&n| n.id).map_or(0, |n| n.id) + 1
    }

    pub(crate) fn move_node(&mut self, x: i8, y: i8) {
        for node in self.floating_nodes.iter_mut() {
            let mut location = node.location;
            location.x = cmp::max(location.x as i32 + x as i32, 0) as u16;
            location.y = cmp::max(location.y as i32 + y as i32, 0) as u16;
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
                self.nodes.append(&mut self.floating_nodes);
                Ok(())
            }
        }
    }

    pub(crate) fn pick(&mut self, name: String) -> Result<()> {
        let matched_node_index = self.nodes.iter_mut().position(|n| n.name == name);
        match matched_node_index {
            Some(index) => {
                let node = self.nodes.remove(index);
                self.floating_nodes.push(node);
            }
            None => return Err(anyhow!("Node with this name {:?} does not exist.", name)),
        }
        Ok(())
    }

    pub(crate) fn delete(&mut self) {
        self.floating_nodes.clear();
    }

    pub(crate) fn overwrite(&mut self, new_node: String) -> Result<()> {
        let new_node: Node = serde_json::from_str(&new_node)?;
        self.floating_nodes[0] = new_node;
        Ok(())
    }

    pub(crate) fn get_floating_serialized(&self) -> Result<String> {
        match self.floating_nodes.len() {
            1 => Ok(serde_json::to_string_pretty(&self.floating_nodes[0])?),
            0 => Err(anyhow!("Tried to serialize with empty floating_nodes.")),
            _ => Err(anyhow!("Tried to serialize multiple floating nodes.")),
        }
    }

    pub(crate) fn connect(&mut self, connection: &Connection) -> Result<()> {
        match self.floating_nodes.len() {
            0 => Err(anyhow!("Tried to connect with empty floating_nodes."))?,
            _ => {
                if !self.nodes.iter().any(|n| n.name == connection.other) {
                    Err(anyhow!("Other `{:?}` does not exist.", connection.other))?
                };
                for node in self.floating_nodes.iter_mut() {
                    node.add_connection(connection);
                }
            }
        };

        Ok(())
    }

    pub(crate) fn connect_reverse(&mut self, connection: &Connection) -> Result<()> {
        match self.floating_nodes.len() {
            0 => Err(anyhow!("Tried to connect with empty floating_nodes."))?,
            _ => {
                for node in self.floating_nodes.iter() {
                    let other_connection = Connection::new(node.name.clone(), connection.weight);
                    for node in self.nodes.iter_mut().filter(|n| n.name == connection.other) {
                        node.add_connection(&other_connection);
                    }
                }
            }
        };

        Ok(())
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
