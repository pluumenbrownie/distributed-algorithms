use ratatui::buffer::Buffer;
use ratatui::layout::Rect;
use ratatui::prelude::Stylize;
use ratatui::style::Style;
use ratatui::widgets::Widget;
use serde::{Deserialize, Serialize};

use super::Location;

#[derive(Debug, Default, Clone)]
pub(crate) struct NodeWidget {
    node: Node,
    style: Style,
}

#[derive(Debug, Default, Clone, Serialize, Deserialize)]
pub(crate) struct Node {
    pub(crate) name: String,
    pub(crate) id: usize,
    pub(crate) connections: Vec<usize>,
    pub(crate) location: Location,
}

impl Node {}

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
