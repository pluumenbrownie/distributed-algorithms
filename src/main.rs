use anyhow::{Context, Result};
// use crossterm::event::{self, Event, KeyCode, KeyEvent, KeyEventKind};
use ratatui::{
    DefaultTerminal, Frame,
    buffer::Buffer,
    crossterm::event::{self, Event, KeyCode, KeyEvent, KeyEventKind, KeyModifiers},
    layout::{Constraint, Flex, Layout, Rect},
    style::{Style, Stylize},
    symbols::border,
    text::Line,
    widgets::{Block, Clear, Widget},
};
use serde::{Deserialize, Serialize};
use std::{
    env, fs,
    io::{self, Write},
    path::{Path, PathBuf},
};
use tui_textarea::TextArea;

use location::Location;
use node::Node;

mod location;
mod node;

const NODE_HEIGHT: u16 = 3;
const NODE_WIDTH: u16 = 6;
const NODE_H_SPACING: u16 = 4;
const NODE_V_SPACING: u16 = 2;

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
enum PopupState {
    #[default]
    Off,
    Save,
    Load,
    Small,
    Large,
}

#[derive(Debug, Default, Clone, Copy)]
enum PopupSize {
    #[default]
    Small = 3,
    Large = 20,
}

impl PopupState {
    fn size(self) -> PopupSize {
        match self {
            Self::Save => PopupSize::Small,
            Self::Load => PopupSize::Small,
            Self::Small => PopupSize::Small,
            Self::Large => PopupSize::Large,
            _ => panic!("Got property of illegal state."),
        }
    }

    fn title_top<'a>(self) -> Line<'a> {
        match self {
            Self::Save => Line::from(" Save structure to... ").left_aligned(),
            Self::Load => Line::from(" Load structure ... ").left_aligned(),
            Self::Small => Line::from(" Small Popup ").left_aligned(),
            Self::Large => Line::from(" Large Popup ").left_aligned(),
            _ => panic!("Got property of illegal state."),
        }
    }

    fn title_bottom<'a>(self) -> Line<'a> {
        match self {
            Self::Save => Line::from(" <Esc> Cancel - <Enter> Save ").right_aligned(),
            Self::Load => Line::from(" <Esc> Cancel - <Enter> Load ").right_aligned(),
            Self::Small => Line::from(" Close with <Esc> ").right_aligned(),
            Self::Large => Line::from(" Close with <Esc> ").right_aligned(),
            _ => panic!("Got property of illegal state."),
        }
    }

    fn content_default(self, latest_dir: &PathBuf, latest_file: &String) -> String {
        match self {
            Self::Save => {
                let mut full_file = latest_dir.clone();
                full_file.push(latest_file);
                full_file.display().to_string()
            }
            Self::Load => {
                let mut full_file = latest_dir.clone();
                full_file.push(latest_file);
                full_file.display().to_string()
            }
            Self::Small => String::from(""),
            Self::Large => String::from(""),
            _ => panic!("Got property of illegal state."),
        }
    }
}

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
    // show_grid: bool,
    show_popup: PopupState,
    textarea: TextArea<'a>,
    latest_dir: PathBuf,
    latest_file: String,
}

fn main() -> Result<()> {
    let mut terminal = ratatui::init();
    let grid = NodeGrid::default();
    let node_display = NodeGridDisplay::new(grid.clone());
    let mut app = App {
        node_display,
        latest_dir: env::current_dir()?,
        latest_file: String::from("grid.json"),
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
        match self.show_popup {
            PopupState::Off => {
                match event::read()? {
                    Event::Key(key_event) if key_event.kind == KeyEventKind::Press => {
                        self.handle_key_event(key_event)?
                    }
                    _ => {}
                };
            }
            PopupState::Save => self.save_textarea()?,
            PopupState::Load => self.load_textarea()?,
            _ => {
                self.handle_textarea()?;
            }
        }
        Ok(())
    }

    fn handle_textarea(&mut self) -> Result<(), anyhow::Error> {
        match event::read()? {
            Event::Key(key_event) if key_event.kind == KeyEventKind::Press => {
                match key_event.code {
                    KeyCode::Esc => self.close_popup(),
                    _ => {
                        self.textarea.input(key_event);
                    }
                }
            }
            _ => {}
        };
        Ok(())
    }

    fn save_textarea(&mut self) -> Result<()> {
        match event::read()? {
            Event::Key(key_event) if key_event.kind == KeyEventKind::Press => {
                match key_event.code {
                    KeyCode::Esc => self.close_popup(),
                    KeyCode::Enter => {
                        let mut path: PathBuf = self.textarea.lines()[0].parse()?;
                        self.save_grid(&path)?;
                        self.latest_file = path.pop().to_string();
                        self.latest_dir = path;
                        self.close_popup();
                    }
                    _ => {
                        self.textarea.input(key_event);
                    }
                }
            }
            _ => {}
        };
        Ok(())
    }

    fn load_textarea(&mut self) -> Result<()> {
        match event::read()? {
            Event::Key(key_event) if key_event.kind == KeyEventKind::Press => {
                match key_event.code {
                    KeyCode::Esc => self.close_popup(),
                    KeyCode::Enter => {
                        let mut path: PathBuf = self.textarea.lines()[0].parse()?;
                        self.load_grid(&path)?;
                        self.latest_file = path
                            .file_name()
                            .expect("I mean I didn't WANT to unwrap here...")
                            .to_string_lossy()
                            .to_string();
                        path.pop();
                        self.latest_dir = path;
                        self.close_popup();
                    }
                    _ => {
                        self.textarea.input(key_event);
                    }
                }
            }
            _ => {}
        };
        Ok(())
    }

    fn handle_key_event(&mut self, key_event: KeyEvent) -> Result<()> {
        match key_event.code {
            KeyCode::Char('q') => self.exit(),
            KeyCode::Char('s') if key_event.modifiers.contains(KeyModifiers::CONTROL) => {
                self.open_popup(PopupState::Save);
            }
            KeyCode::Char('o') if key_event.modifiers.contains(KeyModifiers::CONTROL) => {
                self.open_popup(PopupState::Load);
            }
            KeyCode::Char('t') => self.open_popup(PopupState::Small),
            KeyCode::Char('y') => self.open_popup(PopupState::Large),
            _ => {}
        }
        Ok(())
    }

    fn load_grid(&mut self, path: &PathBuf) -> Result<()> {
        let file = fs::OpenOptions::new().read(true).open(path)?;
        let reader = io::BufReader::new(file);
        self.node_display.grid = serde_json::from_reader(reader)?;

        Ok(())
    }

    fn save_grid(&self, path: &PathBuf) -> Result<()> {
        let file = fs::OpenOptions::new()
            .create(true)
            .truncate(true)
            .write(true)
            .open(path)?;
        let mut writer = io::BufWriter::new(file);
        serde_json::to_writer_pretty(&mut writer, &self.node_display.grid)?;
        writer.flush()?;
        Ok(())
    }

    fn exit(&mut self) {
        self.exit = true;
    }

    fn open_popup(&mut self, state: PopupState) {
        if state == PopupState::Off {
            panic!("")
        }
        self.textarea = TextArea::new(vec![
            state.content_default(&self.latest_dir, &self.latest_file),
        ]);
        self.textarea.set_block(
            Block::bordered()
                .title_top(state.title_top())
                .title_bottom(state.title_bottom()),
        );

        // Move cursor to the end of the text
        self.textarea
            .input(KeyEvent::new(KeyCode::End, KeyModifiers::CONTROL));
        self.show_popup = state;
    }

    fn close_popup(&mut self) {
        self.show_popup = PopupState::Off;
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

        match self.show_popup {
            PopupState::Off => (),
            size => match size.size() {
                PopupSize::Small => {
                    let area = popup_area_small(area, 60, 3);
                    Clear.render(area, buf);
                    self.textarea.render(area, buf);
                }
                PopupSize::Large => {
                    let area = popup_area(area, 60, 20);
                    Clear.render(area, buf);
                    self.textarea.render(area, buf);
                }
            },
        }
    }
}

fn popup_area_small(area: Rect, percent_x: u16, length_y: u16) -> Rect {
    let vertical = Layout::vertical([Constraint::Length(length_y)]).flex(Flex::Center);
    let horizontal = Layout::horizontal([Constraint::Percentage(percent_x)]).flex(Flex::Center);
    let [area] = vertical.areas(area);
    let [area] = horizontal.areas(area);
    area
}

fn popup_area(area: Rect, percent_x: u16, percent_y: u16) -> Rect {
    let vertical = Layout::vertical([Constraint::Percentage(percent_y)]).flex(Flex::Center);
    let horizontal = Layout::horizontal([Constraint::Percentage(percent_x)]).flex(Flex::Center);
    let [area] = vertical.areas(area);
    let [area] = horizontal.areas(area);
    area
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
mod tests;
