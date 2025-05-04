use std::{
    fs,
    io::{self, Write},
};

use crossterm::event::{self, Event, KeyCode, KeyEvent, KeyEventKind};
use location::Location;
use node::Node;
use ratatui::{
    DefaultTerminal, Frame,
    buffer::Buffer,
    layout::Rect,
    style::{Style, Stylize},
    symbols::border,
    text::Line,
    widgets::{Block, Widget},
};
use serde::{Deserialize, Serialize};

use anyhow::{Context, Result};

mod location;
mod node;

const NODE_HEIGHT: u16 = 3;
const NODE_WIDTH: u16 = 6;
const NODE_H_SPACING: u16 = 4;
const NODE_V_SPACING: u16 = 2;

#[derive(Debug, Default, Clone, Serialize, Deserialize)]
pub struct NodeGrid {
    nodes: Vec<node::Node>,
}

#[derive(Debug, Default, Clone)]
pub struct NodeGridDisplay<'a> {
    grid: NodeGrid,
    block: Option<Block<'a>>,
}

#[derive(Debug, Default)]
pub struct App<'a> {
    exit: bool,
    node_display: NodeGridDisplay<'a>,
    show_grid: bool,
}

fn main() -> Result<()> {
    let mut terminal = ratatui::init();
    let grid = NodeGrid::default();
    let node_display = NodeGridDisplay::new(grid.clone());
    let mut app = App {
        node_display,
        ..Default::default()
    };
    let app_result = app.run(&mut terminal);
    ratatui::restore();
    app_result
}

impl App<'_> {
    pub fn run(&mut self, terminal: &mut DefaultTerminal) -> Result<()> {
        while !self.exit {
            terminal
                .draw(|frame| self.draw(frame))
                .context("Drawing to terminal failed.")?;
            self.handle_events()?;
        }
        Ok(())
    }

    fn draw(&self, frame: &mut Frame) {
        frame.render_widget(self, frame.area());
    }

    fn handle_events(&mut self) -> Result<()> {
        match event::read()? {
            Event::Key(key_event) if key_event.kind == KeyEventKind::Press => {
                self.handle_key_event(key_event)?
            }
            _ => {}
        };
        Ok(())
    }

    fn handle_key_event(&mut self, key_event: KeyEvent) -> Result<()> {
        match key_event.code {
            KeyCode::Char('q') => self.exit(),
            KeyCode::Char('s') => self.save_grid()?,
            KeyCode::Char('l') => self.load_grid()?,
            _ => {}
        }
        Ok(())
    }

    fn load_grid(&mut self) -> Result<()> {
        let file = fs::OpenOptions::new().read(true).open("grids/grid.json")?;
        let reader = io::BufReader::new(file);
        self.node_display.grid = serde_json::from_reader(reader)?;

        Ok(())
    }

    fn save_grid(&self) -> Result<()> {
        if !fs::exists("grids")? {
            fs::create_dir("grids")?;
        };
        let file = fs::OpenOptions::new()
            .create(true)
            .truncate(true)
            .write(true)
            .open("grids/grid.json")?;
        let mut writer = io::BufWriter::new(file);
        serde_json::to_writer_pretty(&mut writer, &self.node_display.grid)?;
        writer.flush()?;
        Ok(())
    }

    fn exit(&mut self) {
        self.exit = true;
    }
}

impl Widget for &App<'_> {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let title = Line::from(" Node View ".bold());
        let instructions = Line::from(vec![
            " Save grid ".into(),
            "<S>".blue().bold(),
            " Load grid ".into(),
            "<L>".blue().bold(),
            " Quit ".into(),
            "<Q> ".blue().bold(),
        ]);

        let block_style = Style::default();
        let block = Block::bordered()
            .title(title.centered())
            .title_bottom(instructions.centered())
            .border_style(block_style)
            .border_set(border::THICK);

        self.node_display.clone().block(block).render(area, buf);
    }
}

impl NodeGrid {
    fn new(nodes: Vec<Location>) -> Self {
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

    fn place(&self, node: &Node) -> (u16, u16) {
        (
            NODE_H_SPACING + node.location.horizontal * (NODE_H_SPACING + NODE_WIDTH),
            NODE_V_SPACING + node.location.vertical * (NODE_V_SPACING + NODE_HEIGHT),
        )
    }
}

impl Widget for NodeGrid {
    fn render(self, area: Rect, buf: &mut Buffer)
    where
        Self: Sized,
    {
        for node in self.nodes.iter() {
            let (x, y) = self.place(node);
            let area = Rect::new(x, y, NODE_WIDTH, NODE_HEIGHT);
            node.clone().render(area, buf);
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

#[cfg(test)]
mod tests {
    use super::*;
    // use ratatui::style::Style;

    // #[test]
    // fn render() {
    //     let app = App::default();
    //     let mut buf = Buffer::empty(Rect::new(0, 0, 50, 4));

    //     app.render(buf.area, &mut buf);

    //     let mut expected = Buffer::with_lines(vec![
    //         "┏━━━━━━━━━━━━━ Counter App Tutorial ━━━━━━━━━━━━━┓",
    //         "┃                    Value: 0                    ┃",
    //         "┃                                                ┃",
    //         "┗━ Decrement <Left> Increment <Right> Quit <Q> ━━┛",
    //     ]);
    //     let title_style = Style::new().bold();
    //     let counter_style = Style::new().yellow();
    //     let key_style = Style::new().blue().bold();
    //     expected.set_style(Rect::new(14, 0, 22, 1), title_style);
    //     expected.set_style(Rect::new(28, 1, 1, 1), counter_style);
    //     expected.set_style(Rect::new(13, 3, 6, 1), key_style);
    //     expected.set_style(Rect::new(30, 3, 7, 1), key_style);
    //     expected.set_style(Rect::new(43, 3, 4, 1), key_style);

    //     assert_eq!(buf, expected);
    // }

    #[test]
    fn location_compares() -> io::Result<()> {
        assert!(Location::new(2, 2) < Location::new(3, 3));
        assert!(
            Location::new(2, 2)
                .partial_cmp(&Location::new(3, 3))
                .is_some_and(|x| x.is_lt())
        );
        assert!(Location::new(2, 2) < Location::new(2, 3));
        assert!(Location::new(2, 2) < Location::new(3, 2));
        assert!(Location::new(2, 2) == Location::new(2, 2));
        assert!(Location::new(2, 2) > Location::new(2, 1));
        assert!(Location::new(2, 2) > Location::new(1, 2));
        assert!(
            Location::new(0, 3)
                .partial_cmp(&Location::new(1, 2))
                .is_none()
        );
        assert!(
            Location::new(1, 2)
                .partial_cmp(&Location::new(0, 3))
                .is_none()
        );

        Ok(())
    }

    #[test]
    fn handle_key_event() -> io::Result<()> {
        let mut app = App::default();
        app.handle_key_event(KeyCode::Char('q').into());
        assert!(app.exit);

        Ok(())
    }
}
