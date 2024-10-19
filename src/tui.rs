use anyhow::Result;

use crossterm::event::{self, Event, KeyCode, KeyEvent, KeyEventKind};

use ratatui::{
    prelude::*,
    widgets::{block::Title, Block, Paragraph},
};

use crate::{
    binds::Binds,
    canvas::Canvas,
    config::{Action, Config},
    line::Line,
    rect::Rect,
    text::Text,
    vec::{IVec, UVec},
};

#[derive(Default, Debug)]
enum Mode {
    #[default]
    Normal,

    Rect(Rect),
    Line(Line),
    Text(Text),

    SelectRect {
        cursor_start: UVec,
        original: Rect,
        current: Rect,
    },
}

#[derive(Default)]
struct App {
    binds: Binds,
    cursor: UVec,
    canvas: Canvas,
    exit: bool,
    mode: Mode,
    path: std::path::PathBuf,
    undo_cursor_pos: Vec<UVec>,
    redo_cursor_pos: Vec<UVec>,
    last_edit_cursor_pos: UVec,
}

impl App {
    fn new(config: Config, path: std::path::PathBuf) -> Result<Self> {
        let canvas = if std::fs::exists(&path)? {
            log::debug!("Loading from {path:?}");
            let content = std::fs::read_to_string(&path)?;
            log::trace!("Loading content:\n{content:?}");
            Canvas::from_str(&content)
        } else {
            log::debug!("Creating new canvas");
            Canvas::new(32, 32)
        };
        let binds = Binds::from_config(config.binds)?;
        log::trace!("Using binds: {binds:#?}");
        Ok(Self {
            path,
            binds,
            canvas,
            ..Default::default()
        })
    }

    fn run(&mut self, mut terminal: ratatui::DefaultTerminal) -> Result<()> {
        while !self.exit {
            terminal.draw(|frame| self.draw(frame))?;
            self.handle_events()?;
        }
        Ok(())
    }

    fn draw(&self, frame: &mut Frame) {
        frame.render_widget(self, frame.area());
        // +1 to accomodate border size
        frame.set_cursor_position((self.cursor.x + 1, self.cursor.y + 1));
    }

    fn handle_events(&mut self) -> Result<()> {
        match event::read()? {
            // it's important to check that the event is a key press event as
            // crossterm also emits key release and repeat events on Windows.
            Event::Key(key_event) if key_event.kind == KeyEventKind::Press => {
                self.handle_key_event(key_event)?
            }
            _ => {}
        };
        Ok(())
    }

    fn move_cursor(&mut self, x: i16, y: i16) {
        self.cursor.x = self.cursor.x.saturating_add_signed(x);
        self.cursor.y = self.cursor.y.saturating_add_signed(y);
        log::debug!("Moved cursor to ({:?})", self.cursor);
        match &mut self.mode {
            Mode::Normal => {}
            Mode::Rect(r) => {
                r.bottom_right = self.cursor;
                log::debug!("Updated rect to {r:?}");
            }
            Mode::Line(l) => {
                l.end = self.cursor;
                log::debug!("Updated line to {l:?}");
            }
            Mode::Text(_) => {}
            Mode::SelectRect { current, .. } => {
                *current = current.translated(IVec { x, y });
                log::debug!("Translated rect to {current:?}");
            }
        }
    }

    fn apply_action(&mut self, action: Action) -> Result<()> {
        match action {
            Action::Quit => {
                log::info!("Exit requested");
                self.exit = true;
            }

            Action::Save => {
                log::info!("Saving to {:?}", self.path);
                std::fs::write(&self.path, self.canvas.to_string())?;
            }

            Action::MoveCursorUp => self.move_cursor(0, -1),
            Action::MoveCursorDown => self.move_cursor(0, 1),
            Action::MoveCursorLeft => self.move_cursor(-1, 0),
            Action::MoveCursorRight => self.move_cursor(1, 0),

            Action::DrawRect => {
                self.mode = Mode::Rect(Rect {
                    top_left: self.cursor,
                    bottom_right: self.cursor,
                });
                self.move_cursor(1, 1);
                log::debug!("Set mode: {:?}", self.mode);
            }
            Action::DrawLine => {
                self.mode = Mode::Line(Line::new(self.cursor, self.cursor));
                log::debug!("Set mode: {:?}", self.mode);
            }
            Action::DrawText => {
                self.mode = Mode::Text(Text {
                    start: self.cursor,
                    text: "".into(),
                });
                log::debug!("Set mode: {:?}", self.mode);
            }

            Action::LineAddPoint => match &mut self.mode {
                Mode::Line(l) => {
                    log::debug!("Adding point to line: {l:?}");
                    self.canvas.edit(l.edits().into_iter());
                    self.undo_cursor_pos.push(l.start);
                    self.redo_cursor_pos.clear();
                    self.last_edit_cursor_pos = self.cursor;
                    self.mode = Mode::Line(Line::new(l.end, l.end));
                }
                _ => {}
            },

            Action::LineMirror => match &mut self.mode {
                Mode::Line(l) => {
                    log::debug!("Mirroring line: {l:?}");
                    l.mirror = !l.mirror;
                }
                _ => {}
            },

            Action::ExitMode => match &self.mode {
                Mode::Normal => {}
                Mode::Rect(r) => {
                    log::debug!("Confirming rect {r:?}");
                    self.canvas.edit(r.edits().into_iter());
                    self.undo_cursor_pos.push(r.top_left);
                    self.redo_cursor_pos.clear();
                    self.last_edit_cursor_pos = self.cursor;
                    self.mode = Mode::Normal;
                }
                Mode::Line(l) => {
                    log::debug!("Confirming line {l:?}");
                    self.canvas.edit(l.edits().into_iter());
                    self.undo_cursor_pos.push(l.start);
                    self.redo_cursor_pos.clear();
                    self.last_edit_cursor_pos = self.cursor;
                    self.mode = Mode::Normal;
                }
                Mode::Text(t) => {
                    log::debug!("Confirming text {t:?}");
                    self.canvas.edit(t.edits().into_iter());
                    self.undo_cursor_pos.push(t.start);
                    self.redo_cursor_pos.clear();
                    self.last_edit_cursor_pos = self.cursor;
                    self.mode = Mode::Normal;
                }
                Mode::SelectRect {
                    cursor_start,
                    original,
                    current,
                } => {
                    log::debug!("Deselecing rect {current:?}");
                    self.canvas.edit(
                        original
                            .edits()
                            .into_iter()
                            .map(|e| e.erase())
                            .chain(current.edits().into_iter()),
                    );
                    self.undo_cursor_pos.push(*cursor_start);
                    self.redo_cursor_pos.clear();
                    self.last_edit_cursor_pos = self.cursor;
                    self.mode = Mode::Normal;
                }
            },

            Action::TextAddLine => todo!(),
            Action::Delete => match &self.mode {
                Mode::Normal => {
                    log::debug!("Deleting char at: {:?}", self.cursor);
                    self.canvas.clear(self.cursor);
                }
                Mode::SelectRect {
                    cursor_start,
                    original,
                    ..
                } => {
                    log::debug!("Deleting rect {original:?}");
                    self.canvas
                        .edit(original.edits().into_iter().map(|e| e.erase()));
                    self.undo_cursor_pos.push(*cursor_start);
                    self.redo_cursor_pos.clear();
                    self.last_edit_cursor_pos = self.cursor;
                    self.mode = Mode::Normal;
                }
                mode => {
                    log::debug!("Ignoring delete in mode: {mode:?}");
                }
            },

            Action::Undo => {
                log::debug!("Undo");
                self.canvas.undo();
                if let Some(pos) = self.undo_cursor_pos.pop() {
                    log::debug!("Restoring cursor to {pos:?}");
                    self.redo_cursor_pos.push(pos);
                    self.cursor = pos;
                }
            }
            Action::Redo => {
                log::debug!("Redo");
                self.canvas.redo();
                if let Some(pos) = self.redo_cursor_pos.pop() {
                    log::debug!("Restoring cursor to {pos:?}");
                    self.undo_cursor_pos.push(self.cursor);
                    self.cursor = pos;
                }
            }

            Action::SelectRect => {
                if let Some(rect) = self.canvas.rect_around(self.cursor) {
                    log::info!("Selected rect {rect:?}");
                    self.mode = Mode::SelectRect {
                        cursor_start: self.cursor,
                        original: rect,
                        current: rect,
                    };
                } else {
                    log::info!("No rect matched at {:?}", self.cursor);
                }
            }
        }
        Ok(())
    }

    fn handle_key_event(&mut self, key: KeyEvent) -> Result<()> {
        log::trace!("Handling key {key:?} in mode {:?}", self.mode);

        if let Mode::Text(s) = &mut self.mode {
            match key.code {
                KeyCode::Backspace => {
                    let c = s.text.pop();
                    log::debug!("Popped {c:?} from {s:?}");
                    if c.is_some() {
                        self.move_cursor(-1, 0);
                    }
                    return Ok(());
                }
                KeyCode::Char(c) if key.modifiers.is_empty() => {
                    log::debug!("Appending {c} to {s:?}");
                    s.text.push(c);
                    self.move_cursor(1, 0);
                    return Ok(());
                }
                KeyCode::Enter if key.modifiers.is_empty() => {
                    log::debug!("Appending newline to {s:?}");
                    let len = s.text.lines().last().map(|l| l.len()).unwrap_or(0);
                    s.text.push('\n');
                    self.move_cursor(-(len as i16), 1);
                    return Ok(());
                }
                _ => {}
            }
        }

        let Some(bound) = self.binds.get(&key) else {
            log::trace!("Mapped key to no action");
            return Ok(());
        };
        log::trace!("Mapped key to {bound:?}");

        match bound {
            crate::config::Binding::Single(s) => self.apply_action(s.clone())?,
            crate::config::Binding::Multi(m) => {
                for action in m.clone() {
                    self.apply_action(action)?;
                }
            }
        }
        Ok(())
    }
}

impl Widget for &App {
    fn render(self, area: ratatui::prelude::Rect, buf: &mut Buffer) {
        let title = Title::from("Boxt".bold());
        let instructions = Title::from(ratatui::text::Line::from(vec![
            " Move ".into(),
            "<WASD>".blue().bold(),
            " Rect ".into(),
            "<R>".blue().bold(),
            " Quit ".into(),
            "<Q> ".blue().bold(),
        ]));
        let block = Block::bordered()
            .title(title.alignment(Alignment::Center))
            .title(
                instructions
                    .alignment(Alignment::Center)
                    .position(ratatui::widgets::block::Position::Bottom),
            )
            .border_set(ratatui::symbols::border::THICK);

        // TODO: have separate scratch layer
        let mut canvas = self.canvas.clone();

        let mut style = ratatui::style::Style::default();
        match &self.mode {
            Mode::Normal => {}
            Mode::Rect(r) => {
                log::debug!("Drawing rect: {r:?}");
                canvas.edit(r.edits().into_iter());
            }
            Mode::Line(l) => {
                log::debug!("Drawing line: {l:?}");
                canvas.edit(l.edits().into_iter());
            }
            Mode::Text(t) => {
                log::debug!("Drawing text: {t:?}");
                canvas.edit(t.edits().into_iter());
            }
            Mode::SelectRect {
                original, current, ..
            } => {
                log::debug!("Drawing selected rect: {current:?}");
                canvas.edit(original.edits().into_iter().map(|e| e.erase()));
                canvas.edit(current.edits().into_iter());
                style = style.bold().fg(Color::Cyan);
            }
        }

        let text = ratatui::text::Text::styled(canvas.to_string(), style);
        Paragraph::new(text).block(block).render(area, buf);
    }
}

pub fn start(config: Config, path: std::path::PathBuf) -> Result<()> {
    let mut terminal = ratatui::init();
    terminal.clear()?;

    let app_result = App::new(config, path)?.run(terminal);
    ratatui::restore();
    app_result
}

#[cfg(test)]
mod tests {
    use std::io::Write;

    use super::*;
    use event::KeyModifiers;
    use insta::assert_snapshot;
    use pretty_assertions::assert_eq;

    struct Test {
        app: App,
        tmp: tempfile::NamedTempFile,
    }

    impl Test {
        fn new() -> Test {
            Test::load(&[])
        }

        fn load(lines: &[&str]) -> Test {
            let mut tmp = tempfile::NamedTempFile::new().unwrap();
            tmp.write_all(lines.join("\n").as_bytes()).unwrap();
            tmp.flush().unwrap();
            let app = App::new(Config::default(), tmp.path().to_path_buf()).unwrap();
            Test { app, tmp }
        }

        fn render(&self) -> String {
            let mut buf = Buffer::empty(layout::Rect::new(0, 0, 32, 8));
            self.app.render(buf.area, &mut buf);
            buf_string(&buf)
        }

        fn key(&mut self, key: KeyCode) {
            self.app.handle_key_event(key.into()).unwrap();
        }

        fn input(&mut self, keys: &str) {
            let chars: Vec<_> = keys.chars().collect();
            input(&mut self.app, chars.as_slice());
        }
    }

    fn buf_string(buf: &Buffer) -> String {
        buf.content
            .chunks(buf.area.width as usize)
            .map(|line| {
                line.iter()
                    .map(|cell| cell.symbol().to_string())
                    .collect::<Vec<_>>()
                    .join("")
            })
            .collect::<Vec<_>>()
            .join("\n")
    }

    fn input(app: &mut App, keys: &[char]) {
        for c in keys {
            app.handle_key_event(KeyCode::Char(*c).into()).unwrap();
        }
    }

    #[test]
    fn test_tui_render_empty() {
        let test = Test::new();
        assert_snapshot!(test.render());
    }

    #[test]
    fn test_tui_draw_rect() {
        let mut test = Test::new();

        // Draw one rect and confirm it
        test.input("rsd");
        test.key(KeyCode::Esc);

        // Start drawing another rect
        test.input("ddrsd");

        assert_snapshot!(test.render());
    }

    #[test]
    fn test_tui_draw_line() {
        let mut test = Test::new();

        // Draw a line and confirm it
        test.input("lddsss");
        test.key(KeyCode::Esc);

        // Draw a unconfirmed line with multiple points
        test.input("sdddddlwwaa dd ww");

        assert_snapshot!(test.render());
    }

    #[test]
    fn test_tui_draw_text() {
        let mut test = Test::new();

        // Draw some text
        test.input("sdifoox");
        test.app
            .handle_key_event(KeyCode::Backspace.into())
            .unwrap();

        // Add a new line
        test.app.handle_key_event(KeyCode::Enter.into()).unwrap();
        test.input("bar");

        // Exit text mode
        test.app.handle_key_event(KeyCode::Esc.into()).unwrap();

        // Draw some text without exiting text mode
        test.input("ibaz");

        assert_snapshot!(test.render());
    }

    #[test]
    fn test_tui_load() {
        let test = Test::load(&["  --  ", " hello ", " _   _ ", ""]);
        assert_snapshot!(test.render());
    }

    #[test]
    fn test_tui_save() {
        let mut test = Test::new();

        // Draw some text and confirm it
        test.input("isave this");
        test.app.handle_key_event(KeyCode::Esc.into()).unwrap();

        // Save
        test.app
            .handle_key_event(KeyEvent::new(KeyCode::Char('s'), KeyModifiers::CONTROL))
            .unwrap();

        let actual = std::fs::read_to_string(test.tmp.path()).unwrap();
        assert_snapshot!(actual);
    }

    #[test]
    fn test_tui_delete() {
        let mut test = Test::load(&["delete me"]);
        test.input("xddx");
        assert_snapshot!(test.render());
    }

    #[test]
    fn test_tui_undo_redo() {
        let mut test = Test::new();

        // Draw a few rects
        test.input("rsd");
        test.app.handle_key_event(KeyCode::Esc.into()).unwrap();

        test.input("rssddd");
        test.app.handle_key_event(KeyCode::Esc.into()).unwrap();

        test.input("ddrwwwa");
        test.app.handle_key_event(KeyCode::Esc.into()).unwrap();

        test.input("lsaa");
        test.app.handle_key_event(KeyCode::Esc.into()).unwrap();

        for _ in 0..4 {
            eprintln!("undo");
            test.input("u");
            assert_snapshot!(test.render());
        }

        for _ in 0..4 {
            eprintln!("redo");
            test.input("U");
            assert_snapshot!(test.render());
        }
    }

    #[test]
    fn test_move_rect() {
        let mut test = Test::load(&[
            "                ",
            "   +---+        ",
            "   |   |        ",
            "   |   |        ",
            "   +---+        ",
            "                ",
            "                ",
        ]);

        let before = test.render();

        test.input("ssddddmsd");
        test.app.handle_key_event(KeyCode::Esc.into()).unwrap();

        assert_snapshot!(test.render());

        // undo the move
        test.input("u");
        assert_eq!(test.render(), before);
    }

    #[test]
    fn test_delete_rect() {
        let mut test = Test::load(&[
            "                ",
            "   +---+        ",
            "   |   |        ",
            "   |   |        ",
            "   +---+        ",
            "                ",
            "                ",
        ]);

        test.input("ssddddmx");
        test.app.handle_key_event(KeyCode::Esc.into()).unwrap();

        assert_snapshot!(test.render());
    }
}
