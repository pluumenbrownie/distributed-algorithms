use anyhow::Result;
use ratatui::{buffer::Buffer, layout::Rect, prelude::Stylize, style::Style, widgets::Widget};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use super::Location;

#[derive(Debug, Default, Clone)]
pub(crate) struct NodeWidget {
    node: Node,
    style: Style,
}

#[derive(Debug, Clone)]
pub(crate) struct ConnectionWidget {
    pub sprite: ConnectionSprite,
    pub style: Style,
}

impl ConnectionWidget {
    pub fn new(sprite: ConnectionSprite, style: Style) -> Self {
        ConnectionWidget { sprite, style }
    }

    pub fn set_style(&mut self, style: Style) {
        self.style = style;
    }
}

#[derive(Debug, Clone)]
pub enum ConnectionSprite {
    UndirHorizontal,
    UndirVertical,
    UndirDiagULLR,
    UndirDiagLLUR,
    Upwards,
    Downwards,
    Left,
    Right,
    DiagLRUL,
    DiagULLR,
    DiagLLUR,
    DiagURLL,
    Other(String),
}

impl ConnectionSprite {
    pub fn get(self) -> Vec<String> {
        match self {
            ConnectionSprite::UndirHorizontal => vec!["𜹜𜹜𜹜".into()],
            ConnectionSprite::UndirVertical => {
                vec!["┇".into(), "┇".into(), "┇".into()]
            }
            ConnectionSprite::UndirDiagULLR => vec!["𜹙𜹠".into(), " 𜹒𜹍𜹠".into(), "   𜹒𜹴".into()],
            ConnectionSprite::UndirDiagLLUR => vec!["   𜹰𜹖".into(), " 𜹰𜹍𜹑".into(), "𜹨𜹑".into()],
            ConnectionSprite::Other(string) => vec![format!("&{}", string)],
            ConnectionSprite::Downwards => {
                vec!["┇".into(), "┇".into(), "𜸊".into()]
            }
            ConnectionSprite::Upwards => {
                vec!["𜸉".into(), "┇".into(), "┇".into()]
            }
            ConnectionSprite::Left => vec!["🯝𜹜𜹜".into()],
            ConnectionSprite::Right => vec!["𜹜𜹜🯟".into()],
            ConnectionSprite::DiagLRUL => vec!["🡼𜹠".into(), " 𜹒𜹍𜹠".into(), "   𜹒𜹴".into()],
            ConnectionSprite::DiagULLR => vec!["𜹙𜹠".into(), " 𜹒𜹍𜹠".into(), "   𜹒🡾".into()],
            ConnectionSprite::DiagLLUR => vec!["   𜹰🡽".into(), " 𜹰𜹍𜹑".into(), "𜹨𜹑".into()],
            ConnectionSprite::DiagURLL => vec!["   𜹰𜹖".into(), " 𜹰𜹍𜹑".into(), "🡿𜹑".into()],
        }
    }
}

#[derive(Debug, Default, Clone, Serialize, Deserialize)]
pub struct Connection {
    pub other: String,
    pub weight: f64,
}

impl Connection {
    pub fn new(other: String, weight: f64) -> Self {
        Self { other, weight }
    }

    pub fn directed_sprite(&self, start: &Location, end: &Location) -> ConnectionSprite {
        let dx = end.x as i32 - start.x as i32;
        let dy = end.y as i32 - start.y as i32;

        // Diagonal
        if dx == -1 && dy == 1 {
            ConnectionSprite::DiagURLL
        } else if dx == 1 && dy == -1 {
            ConnectionSprite::DiagLLUR
        } else if dx == -1 && dy == -1 {
            ConnectionSprite::DiagLRUL
        } else if dx == 1 && dy == 1 {
            ConnectionSprite::DiagULLR
        }
        // Straight
        else if dx == 1 && dy == 0 {
            ConnectionSprite::Right
        } else if dx.abs() == 1 && dy == 0 {
            ConnectionSprite::Left
        } else if dx == 0 && dy == 1 {
            ConnectionSprite::Downwards
        } else if dx == 0 && dy == -1 {
            ConnectionSprite::Upwards
        } else {
            ConnectionSprite::Other(self.other.clone())
        }
    }

    pub fn undirected_sprite(&self, start: &Location, end: &Location) -> ConnectionSprite {
        let dx = end.x as i32 - start.x as i32;
        let dy = end.y as i32 - start.y as i32;

        if (dx == -1 && dy == 1) || (dx == 1 && dy == -1) {
            ConnectionSprite::UndirDiagLLUR
        } else if (dx == -1 && dy == -1) || (dx == 1 && dy == 1) {
            ConnectionSprite::UndirDiagULLR
        } else if dx.abs() == 1 && dy == 0 {
            ConnectionSprite::UndirHorizontal
        } else if dx == 0 && dy.abs() == 1 {
            ConnectionSprite::UndirVertical
        } else {
            ConnectionSprite::Other(self.other.clone())
        }
    }
}

#[derive(Debug, Default, Clone, Serialize, Deserialize)]
pub(crate) struct Node {
    pub(crate) name: String,
    pub(crate) id: usize,
    pub(crate) connections: Vec<Connection>,
    pub(crate) location: Location,
}

impl Node {
    pub(crate) fn add_connection(&mut self, connection: &Connection) {
        match self
            .connections
            .iter()
            .position(|n| n.other == connection.other)
        {
            Some(index) => self.connections[index] = connection.clone(),
            None => {
                self.connections.push(connection.clone());
            }
        };
    }
}

pub(crate) fn pad(width: u16, output: &mut String) {
    loop {
        if output.len() as u16 >= width {
            break;
        }
        output.insert(0, ' ');
        if output.len() as u16 >= width {
            break;
        }
        output.push(' ');
    }
}

impl NodeWidget {
    pub fn from(node: &Node, style: Style) -> Self {
        NodeWidget {
            node: node.clone(),
            style,
        }
    }
    pub(crate) fn display_id(&self, width: u16) -> String {
        let mut output = format!("{}", self.node.id);
        pad(width, &mut output);
        output
    }

    pub(crate) fn display_name(&self, width: u16) -> String {
        let mut output = self.node.name.clone();
        pad(width, &mut output);
        output
    }
}

impl Widget for NodeWidget {
    fn render(self, area: Rect, buf: &mut Buffer)
    where
        Self: Sized,
    {
        buf.set_string(area.left(), area.top(), "████", self.style);
        buf.set_string(
            area.left(),
            area.top() + 1,
            self.display_name(area.width),
            self.style.reversed().bold(),
        );
        buf.set_string(area.left(), area.top() + 2, "████", self.style);
    }
}

impl Widget for ConnectionWidget {
    fn render(self, area: Rect, buf: &mut Buffer)
    where
        Self: Sized,
    {
        for (content, line) in self.sprite.get().into_iter().zip(0u16..) {
            buf.set_string(area.left(), area.top() + line, content, self.style);
        }
    }
}
