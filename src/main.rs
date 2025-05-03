use std::io;

use crossterm::event::{self, Event, KeyCode, KeyEvent, KeyEventKind};
use ratatui::{
    DefaultTerminal, Frame,
    buffer::Buffer,
    layout::Rect,
    style::{Style, Stylize},
    symbols::border,
    text::Line,
    widgets::{Block, Widget},
};

const NODE_HEIGHT: u16 = 3;
const NODE_WIDTH: u16 = 5;
const NODE_H_SPACING: u16 = 4;
const NODE_V_SPACING: u16 = 2;

#[derive(Debug, Default, Clone, Copy)]
struct Location {
    horizontal: u16,
    vertical: u16,
}

#[derive(Debug, Default, Clone)]
struct Node {
    name: String,
    id: usize,
    connections: Vec<usize>,
    location: Location,
}

#[derive(Debug, Default, Clone)]
pub struct NodeGrid {
    nodes: Vec<Node>,
}

#[derive(Debug, Default)]
pub struct App {
    counter: u8,
    exit: bool,
    grid: NodeGrid,
}

fn main() -> io::Result<()> {
    let mut terminal = ratatui::init();
    let grid = NodeGrid::new(vec![
        Location::new(0, 0),
        Location::new(1, 1),
        Location::new(2, 4),
    ]);
    let mut app = App {
        grid,
        ..Default::default()
    };
    let app_result = app.run(&mut terminal);
    ratatui::restore();
    app_result
}

impl App {
    pub fn run(&mut self, terminal: &mut DefaultTerminal) -> io::Result<()> {
        while !self.exit {
            terminal.draw(|frame| self.draw(frame))?;
            self.handle_events()?;
        }
        Ok(())
    }

    fn draw(&self, frame: &mut Frame) {
        frame.render_widget(self, frame.area());
    }

    fn handle_events(&mut self) -> io::Result<()> {
        match event::read()? {
            Event::Key(key_event) if key_event.kind == KeyEventKind::Press => {
                self.handle_key_event(key_event)
            }
            _ => {}
        };
        Ok(())
    }

    fn handle_key_event(&mut self, key_event: KeyEvent) {
        match key_event.code {
            KeyCode::Char('q') => self.exit(),
            _ => {}
        }
    }

    fn exit(&mut self) {
        self.exit = true;
    }
}

impl Widget for &App {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let title = Line::from(" Node View ".bold());
        let instructions = Line::from(vec![
            " Decrement ".into(),
            "<Left>".blue().bold(),
            " Increment ".into(),
            "<Right>".blue().bold(),
            " Quit ".into(),
            "<Q> ".blue().bold(),
        ]);
        Block::bordered()
            .title(title.centered())
            .title_bottom(instructions.centered())
            .border_set(border::THICK)
            .render(area, buf);

        self.grid.clone().render(area, buf);
    }
}

impl Location {
    fn new(horizontal: u16, vertical: u16) -> Self {
        Location {
            horizontal,
            vertical,
        }
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

impl Widget for Node {
    fn render(self, area: Rect, buf: &mut Buffer)
    where
        Self: Sized,
    {
        let shape = vec!["███", "█████", "███"];
        for (line_nr, characters) in shape.into_iter().enumerate() {
            buf.set_string(
                area.left() + 2,
                area.top() + 2 + line_nr as u16,
                characters,
                Style::default().fg(ratatui::style::Color::Green),
            );
        }
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
    fn handle_key_event() -> io::Result<()> {
        let mut app = App::default();
        app.handle_key_event(KeyCode::Char('q').into());
        assert!(app.exit);

        Ok(())
    }
}
