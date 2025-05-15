use ratatui::{buffer::Buffer, layout::Rect, style::Style, widgets::Widget};
use serde::{Deserialize, Serialize};

use super::super::Location;
use crate::{NODE_H_SPACING, NODE_HEIGHT, NODE_V_SPACING, NODE_WIDTH};

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

    pub fn get_area(&self) -> Rect {
        match self {
            ConnectionSprite::UndirHorizontal => {
                Rect::new(NODE_WIDTH, NODE_HEIGHT / 2, NODE_H_SPACING, 1)
            }
            ConnectionSprite::UndirVertical => {
                Rect::new(NODE_WIDTH / 2, NODE_HEIGHT, 1, NODE_V_SPACING)
            }
            ConnectionSprite::UndirDiagLLUR => Rect::new(NODE_WIDTH - 1, NODE_HEIGHT, 3, 5),
            ConnectionSprite::UndirDiagULLR => Rect::new(NODE_WIDTH - 1, NODE_HEIGHT, 5, 3),
            ConnectionSprite::Left => Rect::new(NODE_WIDTH, NODE_HEIGHT / 2, NODE_H_SPACING, 1),
            ConnectionSprite::Right => Rect::new(NODE_WIDTH, NODE_HEIGHT / 2, NODE_H_SPACING, 1),
            ConnectionSprite::Upwards => Rect::new(NODE_WIDTH / 2, NODE_HEIGHT, 1, NODE_V_SPACING),
            ConnectionSprite::Downwards => {
                Rect::new(NODE_WIDTH / 2, NODE_HEIGHT, 1, NODE_V_SPACING)
            }
            ConnectionSprite::DiagLLUR => Rect::new(NODE_WIDTH - 1, NODE_HEIGHT, 3, 5),
            ConnectionSprite::DiagURLL => Rect::new(NODE_WIDTH - 1, NODE_HEIGHT, 3, 5),
            ConnectionSprite::DiagULLR => Rect::new(NODE_WIDTH - 1, NODE_HEIGHT, 5, 3),
            ConnectionSprite::DiagLRUL => Rect::new(NODE_WIDTH - 1, NODE_HEIGHT, 5, 3),
            ConnectionSprite::Other(_) => Rect::new(NODE_WIDTH / 2 - 1, NODE_HEIGHT, 1, 1),
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
        } else if dx == -1 && dy == 0 {
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

impl Widget for ConnectionWidget {
    fn render(self, area: Rect, buf: &mut Buffer)
    where
        Self: Sized,
    {
        for (content, line) in self.sprite.get().into_iter().zip(0u16..) {
            let white_space = content.chars().filter(|c| c.is_whitespace()).count();
            buf.set_string(
                area.left() + white_space as u16,
                area.top() + line,
                content.trim(),
                self.style,
            );
        }
    }
}
