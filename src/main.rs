#![allow(unused_variables, unused_imports, dead_code)]

use anyhow::{Context, Result, anyhow};
use node::Connection;
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
use std::{
    env, fs,
    io::{self, Write},
    path::{Path, PathBuf},
};
use tui_textarea::TextArea;

use location::Location;

mod location;
mod node;
mod nodegrid;

const NODE_HEIGHT: u16 = 3;
const NODE_WIDTH: u16 = 6;
const NODE_H_SPACING: u16 = 3;
const NODE_V_SPACING: u16 = 3;

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
enum AppState {
    #[default]
    Default,
    Selection,
    Popup(PopupState),
}

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
enum PopupState {
    Save,
    Load,
    New,
    Pick,
    Connect,
    #[default]
    Small,
    Edit,
    Large,
}

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
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
            Self::New => PopupSize::Small,
            Self::Pick => PopupSize::Small,
            Self::Connect => PopupSize::Small,
            Self::Small => PopupSize::Small,
            Self::Edit => PopupSize::Large,
            Self::Large => PopupSize::Large,
        }
    }

    fn title_top<'a>(self) -> Line<'a> {
        match self {
            Self::Save => Line::from(" Save structure to... ").left_aligned(),
            Self::Load => Line::from(" Load structure... ").left_aligned(),
            Self::New => Line::from(" Unique node name ").left_aligned(),
            Self::Pick => Line::from(" Pick node with name ").left_aligned(),
            Self::Connect => Line::from(" Create weighted connection ").left_aligned(),
            Self::Small => Line::from(" Small Popup ").left_aligned(),
            Self::Edit => Line::from(" Edit node ").left_aligned(),
            Self::Large => Line::from(" Large Popup ").left_aligned(),
        }
    }

    fn title_bottom<'a>(self) -> Line<'a> {
        match self {
            Self::Save => Line::from(" <Esc> Cancel - <Enter> Save ").right_aligned(),
            Self::Load => Line::from(" <Esc> Cancel - <Enter> Load ").right_aligned(),
            Self::New => Line::from(" <Esc> Cancel - <Enter> Create ").right_aligned(),
            Self::Pick => Line::from(" <Esc> Cancel - <Enter> Pick ").right_aligned(),
            Self::Connect => {
                Line::from(" <Esc> Cancel - <Enter> Create <Alt+Enter> Create undirected ")
                    .right_aligned()
            }
            Self::Small => Line::from(" Close with <Esc> ").right_aligned(),
            Self::Edit => Line::from(" <Esc> Cancel - <Ctrl+s> Apply ").right_aligned(),
            Self::Large => Line::from(" Close with <Esc> ").right_aligned(),
        }
    }

    fn content_default(self, app: &App) -> String {
        match self {
            Self::Save => {
                let mut full_file = app.latest_dir.to_path_buf();
                full_file.push(app.latest_file.clone());
                full_file.display().to_string()
            }
            Self::Load => {
                let mut full_file = app.latest_dir.to_path_buf();
                full_file.push(app.latest_file.clone());
                full_file.display().to_string()
            }
            Self::New => String::from(""),
            Self::Pick => String::from(""),
            Self::Connect => String::from("name 1.0"),
            Self::Small => String::from(""),
            Self::Edit => app.get_node_serialized(),
            Self::Large => String::from(""),
        }
    }
}

#[derive(Debug, Default)]
pub struct App<'a> {
    exit: bool,
    node_display: nodegrid::NodeGridDisplay<'a>,
    // show_grid: bool,
    state: AppState,
    textarea: TextArea<'a>,
    latest_dir: PathBuf,
    latest_file: String,
}

fn main() -> Result<()> {
    let mut terminal = ratatui::init();
    let grid = nodegrid::NodeGrid::default();
    let node_display = nodegrid::NodeGridDisplay::new(grid.clone());
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
        match self.state {
            AppState::Default => {
                match event::read()? {
                    Event::Key(key_event) if key_event.kind == KeyEventKind::Press => {
                        self.handle_default_key_event(key_event)?
                    }
                    _ => {}
                };
            }
            AppState::Selection => {
                match event::read()? {
                    Event::Key(key_event) if key_event.kind == KeyEventKind::Press => {
                        self.handle_selection_key_event(key_event)?
                    }
                    _ => {}
                };
            }
            AppState::Popup(popup) => match popup {
                PopupState::Save => self.save_textarea()?,
                PopupState::Load => self.load_textarea()?,
                PopupState::New => self.new_textarea()?,
                PopupState::Pick => self.pick_textarea()?,
                PopupState::Edit => self.edit_textarea()?,
                PopupState::Large => {
                    self.handle_textarea_key_event()?;
                }
                PopupState::Small => {
                    self.handle_textarea_key_event()?;
                }
                PopupState::Connect => self.connect_textarea()?,
            },
        }
        Ok(())
    }

    fn handle_default_key_event(&mut self, key_event: KeyEvent) -> Result<()> {
        match key_event.code {
            KeyCode::Char('q') => self.exit(),
            KeyCode::Char('s') if key_event.modifiers.contains(KeyModifiers::CONTROL) => {
                self.open_popup(PopupState::Save);
            }
            KeyCode::Char('o') if key_event.modifiers.contains(KeyModifiers::CONTROL) => {
                self.open_popup(PopupState::Load);
            }
            KeyCode::Char('p') => self.open_popup(PopupState::Pick),
            KeyCode::Char('t') => self.open_popup(PopupState::Small),
            KeyCode::Char('y') => self.open_popup(PopupState::Large),
            KeyCode::Char('n') => self.open_popup(PopupState::New),
            _ => {}
        }
        Ok(())
    }

    fn handle_selection_key_event(&mut self, key_event: KeyEvent) -> Result<()> {
        match key_event.code {
            // In ratatui, down is positive
            KeyCode::Down => self.move_node(0, 1),
            KeyCode::Up => self.move_node(0, -1),
            // and right is positive
            KeyCode::Right => self.move_node(1, 0),
            KeyCode::Left => self.move_node(-1, 0),

            KeyCode::Char('e') => {
                self.open_popup(PopupState::Edit);
            }
            KeyCode::Char('c') => {
                self.open_popup(PopupState::Connect);
            }
            KeyCode::Backspace | KeyCode::Delete => {
                self.delete_selection();
                self.state_default()
            }
            KeyCode::Enter => match self.commit_selection() {
                Err(_) => {}
                Ok(_) => self.state_default(),
            },
            _ => {}
        }
        Ok(())
    }

    fn handle_textarea_key_event(&mut self) -> Result<()> {
        match event::read()? {
            Event::Key(key_event) if key_event.kind == KeyEventKind::Press => {
                match key_event.code {
                    KeyCode::Esc => self.state_default(),
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
        let mut enter_func = |app: &mut App| {
            let path: PathBuf = app.textarea.lines()[0].parse()?;
            app.save_grid(&path)?;
            app.set_latest_location(path);
            app.state_default();
            Ok(())
        };
        self.confirm_cancel_textarea(&mut enter_func)
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

    fn load_textarea(&mut self) -> Result<()> {
        let mut enter_func = |app: &mut App| {
            let path: PathBuf = app.textarea.lines()[0].parse()?;
            app.load_grid(&path)?;
            app.set_latest_location(path);
            app.state_default();
            Ok(())
        };
        self.confirm_cancel_textarea(&mut enter_func)
    }

    fn load_grid(&mut self, path: &PathBuf) -> Result<()> {
        let file = fs::OpenOptions::new().read(true).open(path)?;
        let reader = io::BufReader::new(file);
        self.node_display.grid = serde_json::from_reader(reader)?;

        Ok(())
    }

    fn set_latest_location(&mut self, mut path: PathBuf) {
        self.latest_file = path
            .file_name()
            .expect("I mean I didn't WANT to unwrap here...")
            .to_string_lossy()
            .to_string();
        path.pop();
        self.latest_dir = path;
    }

    fn new_textarea(&mut self) -> Result<()> {
        let mut enter_func = |app: &mut App| {
            let name = app.textarea.lines()[0].clone();
            if app.add_node(name).is_ok() {
                app.state_default();
                app.state = AppState::Selection;
            }
            Ok(())
        };
        self.confirm_cancel_textarea(&mut enter_func)
    }

    fn pick_textarea(&mut self) -> Result<()> {
        let mut enter_func = |app: &mut App| {
            let name = app.textarea.lines()[0].clone();
            if app.pick_node(name).is_ok() {
                app.state_default();
                app.state = AppState::Selection;
            }
            Ok(())
        };
        self.confirm_cancel_textarea(&mut enter_func)
    }

    /// Function for text areas which only need to run a function if Enter is pressed,
    /// or to close when Esc is pressed. Else, just type in the text area.
    fn confirm_cancel_textarea<F>(&mut self, mut enter_func: F) -> Result<()>
    where
        F: FnMut(&mut Self) -> Result<()>,
    {
        match event::read()? {
            Event::Key(key_event) if key_event.kind == KeyEventKind::Press => {
                match key_event.code {
                    KeyCode::Esc => self.state_default(),
                    KeyCode::Enter => enter_func(self)?,
                    _ => {
                        self.textarea.input(key_event);
                    }
                }
            }
            _ => {}
        };
        Ok(())
    }

    fn edit_textarea(&mut self) -> Result<()> {
        match event::read()? {
            Event::Key(key_event) if key_event.kind == KeyEventKind::Press => {
                match key_event.code {
                    KeyCode::Esc => self.state = AppState::Selection,
                    KeyCode::Char('s') if key_event.modifiers.contains(KeyModifiers::CONTROL) => {
                        let new_node = self.textarea.lines().concat();
                        if self.overwrite_selection(new_node).is_ok() {
                            self.state = AppState::Selection;
                        }
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

    fn connect_textarea(&mut self) -> Result<()> {
        match event::read()? {
            Event::Key(key_event) if key_event.kind == KeyEventKind::Press => {
                match key_event.code {
                    KeyCode::Esc => self.state = AppState::Selection,
                    KeyCode::Enter => {
                        let input = self.textarea.lines()[0].split_once(" ").ok_or(anyhow!(
                            "Bad connection for connection: {:?}",
                            self.textarea.lines()[0]
                        ))?;
                        let connection =
                            Connection::new(input.0.to_string(), input.1.parse::<f64>()?);
                        if self.connect_selection(&connection).is_ok() {
                            if key_event.modifiers.contains(KeyModifiers::ALT) {
                                if self.connect_other(&connection).is_ok() {
                                    self.state = AppState::Selection;
                                }
                            } else {
                                self.state = AppState::Selection;
                            }
                        }
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

    fn exit(&mut self) {
        self.exit = true;
    }

    fn open_popup(&mut self, state: PopupState) {
        self.textarea = TextArea::from(state.content_default(self).split('\n'));
        self.textarea.set_block(
            Block::bordered()
                .title_top(state.title_top())
                .title_bottom(state.title_bottom()),
        );

        // Move cursor to the end of the text
        self.textarea
            .input(KeyEvent::new(KeyCode::End, KeyModifiers::CONTROL));
        self.state = AppState::Popup(state);
    }

    fn state_default(&mut self) {
        self.state = AppState::Default;
    }

    fn add_node(&mut self, name: String) -> Result<()> {
        self.node_display.grid.new_node(name)
    }

    fn move_node(&mut self, x: i8, y: i8) {
        self.node_display.grid.move_node(x, y)
    }

    fn pick_node(&mut self, name: String) -> Result<()> {
        self.node_display.grid.pick(name)
    }

    fn commit_selection(&mut self) -> Result<()> {
        self.node_display.grid.commit()
    }

    fn delete_selection(&mut self) {
        self.node_display.grid.delete()
    }

    fn overwrite_selection(&mut self, new_node: String) -> Result<()> {
        self.node_display.grid.overwrite(new_node)
    }

    fn get_node_serialized(&self) -> String {
        self.node_display.grid.get_floating_serialized().unwrap()
    }

    fn connect_selection(&mut self, connection: &Connection) -> Result<()> {
        self.node_display.grid.connect(connection)
    }

    fn connect_other(&mut self, connection: &Connection) -> Result<()> {
        self.node_display.grid.connect_reverse(connection)
    }
}

impl Widget for &App<'_> {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let title = Line::from(vec![
            " Current state: ".into(),
            format!("{:?}", self.state).into(),
            " ".into(),
        ]);
        let instructions = match self.state {
            AppState::Default => Line::from(vec![
                " New node ".into(),
                "<N>".blue().bold(),
                " Pick node ".into(),
                "<P>".blue().bold(),
                " Save grid ".into(),
                "<Ctrl+S>".blue().bold(),
                " Load grid ".into(),
                "<Ctrl+O>".blue().bold(),
                " Quit ".into(),
                "<Q> ".blue().bold(),
            ]),
            AppState::Selection => Line::from(vec![
                " Move ".into(),
                "<󰁍󰁅󰁝󰁔>".blue().bold(),
                " Edit ".into(),
                "<E>".blue().bold(),
                " Place node ".into(),
                "<Enter> ".blue().bold(),
            ]),
            AppState::Popup(_) => Line::from(" Follow instructions in popup "),
        };

        let block_style = Style::default();
        let block = Block::bordered()
            .title(title.centered())
            .title_bottom(instructions.centered())
            .border_style(block_style)
            .border_set(border::THICK);

        self.node_display.clone().block(block).render(area, buf);

        match self.state {
            AppState::Default => {}
            AppState::Selection => {}
            AppState::Popup(popup) => match popup.size() {
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

#[cfg(test)]
mod tests;
