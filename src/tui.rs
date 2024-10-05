use anyhow::Result;

use crossterm::event::{self, Event, KeyCode, KeyEvent, KeyEventKind};

use ratatui::{
    prelude::*,
    widgets::{block::Title, Block, Paragraph},
};

use crate::{
    binds::Binds,
    canvas::Canvas,
    config::{Action, Config, EnterMode},
    line::Line,
    point::Point,
    rect::Rect,
    text::Text,
};

#[derive(Default, Debug)]
enum Mode {
    #[default]
    Normal,
    Rect(Rect),
    Line(Line),
    Text(Text),
}

#[derive(Default)]
struct App {
    binds: Binds,
    cursor: Point,
    canvas: Canvas,
    exit: bool,
    mode: Mode,
    path: std::path::PathBuf,
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

    fn warp_cursor(&mut self, p: &Point) {
        self.cursor = *p;
        log::debug!("Moved cursor to {p:?}");
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
                let Some(last) = l.0.last_mut() else {
                    log::warn!("Zero-point line");
                    return;
                };
                *last = self.cursor;
                log::debug!("Updated line to {l:?}");
            }
            Mode::Text(_) => {}
        }
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
                _ => {}
            }
        }

        let Some(action) = self.binds.get(&key) else {
            log::trace!("Mapped key to no action");
            return Ok(());
        };
        log::trace!("Mapped key to action {action:?}");

        match action {
            Action::Quit => {
                log::info!("Exit requested");
                self.exit = true;
            }

            Action::Save => {
                log::info!("Saving to {:?}", self.path);
                std::fs::write(&self.path, self.canvas.to_string())?;
            }

            Action::MoveCursor { x, y } => self.move_cursor(*x, *y),

            Action::EnterMode(mode) => match mode {
                EnterMode::Rect => {
                    self.mode = Mode::Rect(Rect {
                        top_left: self.cursor,
                        bottom_right: self.cursor,
                    });
                    self.move_cursor(1, 1);
                    log::debug!("Set mode: {:?}", self.mode);
                }
                EnterMode::Line => {
                    self.mode = Mode::Line(Line(vec![self.cursor; 2]));
                    log::debug!("Set mode: {:?}", self.mode);
                }
                EnterMode::Text => {
                    self.mode = Mode::Text(Text {
                        start: self.cursor,
                        text: "".into(),
                    });
                    log::debug!("Set mode: {:?}", self.mode);
                }
            },

            Action::LineAddPoint => match &mut self.mode {
                Mode::Line(l) => {
                    log::debug!("Adding point to line: {l:?}");
                    l.0.push(self.cursor);
                }
                _ => {}
            },

            Action::Confirm => match &self.mode {
                Mode::Normal => {}
                Mode::Rect(r) => {
                    log::debug!("Confirming rect {r:?}");
                    r.draw(&mut self.canvas);
                    self.mode = Mode::Normal;
                }
                Mode::Line(l) => {
                    log::debug!("Confirming line {l:?}");
                    l.draw(&mut self.canvas);
                    self.mode = Mode::Normal;
                }
                Mode::Text(t) => {
                    log::debug!("Confirming text {t:?}");
                    t.draw(&mut self.canvas);
                    self.mode = Mode::Normal;
                }
            },

            Action::Cancel => {
                log::debug!("Cancelling mode: {:?}", self.mode);
                match std::mem::take(&mut self.mode) {
                    Mode::Normal => {}
                    Mode::Rect(r) => {
                        self.warp_cursor(&r.top_left);
                    }
                    Mode::Line(l) => {
                        if let Some(p) = l.0.first() {
                            self.warp_cursor(p);
                        }
                    }
                    Mode::Text(t) => {
                        self.warp_cursor(&t.start);
                    }
                }
            }

            _ => {}
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

        match &self.mode {
            Mode::Normal => {}
            Mode::Rect(r) => {
                log::debug!("Drawing rect: {r:?}");
                r.draw(&mut canvas);
            }
            Mode::Line(l) => {
                log::debug!("Drawing line: {l:?}");
                l.draw(&mut canvas);
            }
            Mode::Text(t) => {
                log::debug!("Drawing text: {t:?}");
                t.draw(&mut canvas);
            }
        }

        let text = ratatui::text::Text::raw(canvas.to_string());
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
    fn test_render_empty() {
        let tmp = tempfile::NamedTempFile::new().unwrap();
        let mut app = App::new(Config::default(), tmp.path().to_path_buf()).unwrap();
        let mut buf = Buffer::empty(layout::Rect::new(0, 0, 32, 8));

        app.render(buf.area, &mut buf);

        assert_snapshot!(buf_string(&buf));
    }

    #[test]
    fn test_draw_rect() {
        let tmp = tempfile::NamedTempFile::new().unwrap();
        let mut app = App::new(Config::default(), tmp.path().to_path_buf()).unwrap();
        let mut buf = Buffer::empty(layout::Rect::new(0, 0, 32, 8));

        // Draw one rect and confirm it
        app.handle_key_event(KeyCode::Char('r').into()).unwrap();
        app.handle_key_event(KeyCode::Char('s').into()).unwrap();
        app.handle_key_event(KeyCode::Char('d').into()).unwrap();
        app.handle_key_event(KeyCode::Enter.into()).unwrap();

        // Start drawing another rect
        app.handle_key_event(KeyCode::Char('d').into()).unwrap();
        app.handle_key_event(KeyCode::Char('d').into()).unwrap();
        app.handle_key_event(KeyCode::Char('r').into()).unwrap();
        app.handle_key_event(KeyCode::Char('s').into()).unwrap();
        app.handle_key_event(KeyCode::Char('d').into()).unwrap();

        app.render(buf.area, &mut buf);

        assert_snapshot!(buf_string(&buf));
    }

    #[test]
    fn test_cancel_rect() {
        let tmp = tempfile::NamedTempFile::new().unwrap();
        let mut app = App::new(Config::default(), tmp.path().to_path_buf()).unwrap();
        let mut buf = Buffer::empty(layout::Rect::new(0, 0, 32, 8));

        // Draw one rect and cancel it
        app.handle_key_event(KeyCode::Char('r').into()).unwrap();
        app.handle_key_event(KeyCode::Char('s').into()).unwrap();
        app.handle_key_event(KeyCode::Char('d').into()).unwrap();
        app.handle_key_event(KeyCode::Esc.into()).unwrap();

        // Start drawing another rect
        app.handle_key_event(KeyCode::Char('d').into()).unwrap();
        app.handle_key_event(KeyCode::Char('d').into()).unwrap();
        app.handle_key_event(KeyCode::Char('r').into()).unwrap();
        app.handle_key_event(KeyCode::Char('s').into()).unwrap();
        app.handle_key_event(KeyCode::Char('d').into()).unwrap();

        app.render(buf.area, &mut buf);

        assert_snapshot!(buf_string(&buf));
    }

    #[test]
    fn test_draw_line() {
        let tmp = tempfile::NamedTempFile::new().unwrap();
        let mut app = App::new(Config::default(), tmp.path().to_path_buf()).unwrap();
        let mut buf = Buffer::empty(layout::Rect::new(0, 0, 32, 8));

        // Draw a line and confirm it
        input(&mut app, &['l', 'd', 'd', 's', 's', 's']);
        app.handle_key_event(KeyCode::Enter.into()).unwrap();

        // Draw a line and cancel it
        input(&mut app, &['l', 'd', 'd', 's', 's', 's']);
        app.handle_key_event(KeyCode::Esc.into()).unwrap();

        // Draw a unconfirmed line with multiple points
        input(
            &mut app,
            &[
                's', 'd', 'd', 'd', 'd', 'd', 'l', 'w', 'w', 'a', 'a', ' ', 'd', 'd', ' ', 'w', 'w',
            ],
        );

        app.render(buf.area, &mut buf);

        assert_snapshot!(buf_string(&buf));
    }

    #[test]
    fn test_draw_text() {
        let tmp = tempfile::NamedTempFile::new().unwrap();
        let mut app = App::new(Config::default(), tmp.path().to_path_buf()).unwrap();
        let mut buf = Buffer::empty(layout::Rect::new(0, 0, 32, 8));

        // Draw some text and confirm it
        input(&mut app, &['s', 'd', 'i', 'f', 'o', 'o', 'x']);
        app.handle_key_event(KeyCode::Backspace.into()).unwrap();
        app.handle_key_event(KeyCode::Enter.into()).unwrap();

        // Draw some more text and cancel it
        input(&mut app, &['s', 'd', 'i', 'f', 'o', 'o', 'x']);
        app.handle_key_event(KeyCode::Esc.into()).unwrap();

        // Draw some unconfirmed text
        input(&mut app, &['i', 'b', 'a', 'r']);

        app.render(buf.area, &mut buf);

        assert_snapshot!(buf_string(&buf));
    }

    #[test]
    fn test_load() {
        let mut tmp = tempfile::NamedTempFile::new().unwrap();
        tmp.write_all("  --  \n hello \n _   _ \n".as_bytes())
            .unwrap();
        tmp.flush().unwrap();
        let app = App::new(Config::default(), tmp.path().to_path_buf()).unwrap();

        let mut buf = Buffer::empty(layout::Rect::new(0, 0, 32, 8));
        app.render(buf.area, &mut buf);

        assert_snapshot!(buf_string(&buf));
    }

    #[test]
    fn test_save() {
        let tmp = tempfile::NamedTempFile::new().unwrap();
        let mut app = App::new(Config::default(), tmp.path().to_path_buf()).unwrap();

        // Draw some text and confirm it
        input(
            &mut app,
            &['i', 's', 'a', 'v', 'e', ' ', 't', 'h', 'i', 's'],
        );
        app.handle_key_event(KeyCode::Enter.into()).unwrap();

        // Save
        app.handle_key_event(KeyEvent::new(KeyCode::Char('s'), KeyModifiers::CONTROL))
            .unwrap();

        let actual = std::fs::read_to_string(tmp.path()).unwrap();
        assert_snapshot!(actual);
    }
}
