use ratatui::buffer::Buffer;
use ratatui::layout::Rect;
use ratatui::prelude::Stylize;
use ratatui::style::Style;
use ratatui::widgets::Widget;
use serde::{Deserialize, Serialize};

use super::Location;

#[derive(Debug, Default, Clone, Serialize, Deserialize)]
pub(crate) struct Node {
    pub(crate) name: String,
    pub(crate) id: usize,
    pub(crate) connections: Vec<usize>,
    pub(crate) location: Location,
}

impl Node {
    pub(crate) fn display_id(&self, width: u16) -> String {
        let mut output = String::new();
        output.push_str(format!("{}", self.id).as_str());
        pad(width, &mut output);
        output
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

impl Widget for Node {
    fn render(self, area: Rect, buf: &mut Buffer)
    where
        Self: Sized,
    {
        buf.set_string(
            area.left(),
            area.top(),
            "████",
            Style::default().fg(ratatui::style::Color::Green),
        );
        buf.set_string(
            area.left(),
            area.top() + 1,
            self.display_id(area.width),
            Style::default()
                .bg(ratatui::style::Color::Green)
                .fg(ratatui::style::Color::Black)
                .bold(),
        );
        buf.set_string(
            area.left(),
            area.top() + 2,
            "████",
            Style::default().fg(ratatui::style::Color::Green),
        );
    }
}
