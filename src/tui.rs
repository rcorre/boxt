use anyhow::Result;

use crossterm::event::{self, Event, KeyCode, KeyEvent, KeyEventKind};

use ratatui::{
    prelude::*,
    widgets::{block::Title, Block, Paragraph},
};

use crate::{canvas::Canvas, point::Point, rect::Rect, text::Text};

#[derive(Default, Debug)]
enum Mode {
    #[default]
    Normal,
    Rect(Rect),
    Text(Text),
}

#[derive(Default)]
struct App {
    cursor_x: u16,
    cursor_y: u16,
    canvas: Canvas,
    exit: bool,
    mode: Mode,
}

impl App {
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
        frame.set_cursor_position((self.cursor_x + 1, self.cursor_y + 1));
    }

    fn handle_events(&mut self) -> Result<()> {
        match event::read()? {
            // it's important to check that the event is a key press event as
            // crossterm also emits key release and repeat events on Windows.
            Event::Key(key_event) if key_event.kind == KeyEventKind::Press => {
                self.handle_key_event(key_event)
            }
            _ => {}
        };
        Ok(())
    }

    fn warp_cursor(&mut self, p: &Point) {
        self.cursor_x = p.x;
        self.cursor_y = p.y;
        log::debug!("Moved cursor to {p:?}");
    }

    fn move_cursor(&mut self, x: i16, y: i16) {
        self.cursor_x = self.cursor_x.saturating_add_signed(x);
        self.cursor_y = self.cursor_y.saturating_add_signed(y);
        log::debug!("Moved cursor to ({}, {})", self.cursor_x, self.cursor_y);
        match &mut self.mode {
            Mode::Normal => {}
            Mode::Rect(r) => {
                r.bottom_right.x = self.cursor_x;
                r.bottom_right.y = self.cursor_y;
                log::debug!("Updated rect to {r:?}");
            }
            Mode::Text(_) => {}
        }
    }

    fn handle_key_event(&mut self, key: KeyEvent) {
        log::trace!("Handling key {:?}", key);

        if let Mode::Text(s) = &mut self.mode {
            match key.code {
                KeyCode::Backspace => {
                    let c = s.text.pop();
                    log::debug!("Popped {c:?} from {s:?}");
                    if c.is_some() {
                        self.move_cursor(-1, 0);
                    }
                    return;
                }
                KeyCode::Char(c) => {
                    log::debug!("Appending {c} to {s:?}");
                    s.text.push(c);
                    self.move_cursor(1, 0);
                    return;
                }
                _ => {}
            }
        }

        match key.code {
            KeyCode::Char('q') => self.exit = true,

            KeyCode::Char('w') => self.move_cursor(0, -1),
            KeyCode::Char('s') => self.move_cursor(0, 1),
            KeyCode::Char('a') => self.move_cursor(-1, 0),
            KeyCode::Char('d') => self.move_cursor(1, 0),

            KeyCode::Char('r') => {
                self.mode = Mode::Rect(Rect::new(self.cursor_x, self.cursor_y, 0, 0));
                self.move_cursor(1, 1);
                log::debug!("Set mode: {:?}", self.mode);
            }
            KeyCode::Char('i') => {
                self.mode = Mode::Text(Text::new(self.cursor_x, self.cursor_y, ""));
                log::debug!("Set mode: {:?}", self.mode);
            }

            KeyCode::Enter => match &self.mode {
                Mode::Normal => {}
                Mode::Rect(r) => {
                    log::debug!("Confirming rect {:?}", r);
                    r.draw(&mut self.canvas);
                    self.mode = Mode::Normal;
                }
                Mode::Text(t) => {
                    log::debug!("Confirming text {:?}", t);
                    t.draw(&mut self.canvas);
                    self.mode = Mode::Normal;
                }
            },

            KeyCode::Esc => {
                log::debug!("Cancelling mode: {:?}", self.mode);
                match std::mem::take(&mut self.mode) {
                    Mode::Normal => {}
                    Mode::Rect(r) => {
                        self.warp_cursor(&r.top_left);
                    }
                    Mode::Text(t) => {
                        self.warp_cursor(&t.start);
                    }
                }
            }

            _ => {}
        }
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
            Mode::Text(t) => {
                log::debug!("Drawing text: {t:?}");
                t.draw(&mut canvas);
            }
        }

        let text = ratatui::text::Text::raw(canvas.to_string());
        Paragraph::new(text).block(block).render(area, buf);
    }
}

pub fn start() -> Result<()> {
    let mut terminal = ratatui::init();
    terminal.clear()?;

    let app_result = App::default().run(terminal);
    ratatui::restore();
    app_result
}

#[cfg(test)]
mod tests {
    use super::*;
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

    #[test]
    fn test_render_empty() {
        let app = App::default();
        let mut buf = Buffer::empty(layout::Rect::new(0, 0, 32, 8));

        app.render(buf.area, &mut buf);

        assert_snapshot!(buf_string(&buf));
    }

    #[test]
    fn test_draw_rect() {
        let mut app = App::default();
        let mut buf = Buffer::empty(layout::Rect::new(0, 0, 32, 8));

        // Draw one rect and confirm it
        app.handle_key_event(KeyCode::Char('r').into());
        app.handle_key_event(KeyCode::Char('s').into());
        app.handle_key_event(KeyCode::Char('d').into());
        app.handle_key_event(KeyCode::Enter.into());

        // Start drawing another rect
        app.handle_key_event(KeyCode::Char('d').into());
        app.handle_key_event(KeyCode::Char('d').into());
        app.handle_key_event(KeyCode::Char('r').into());
        app.handle_key_event(KeyCode::Char('s').into());
        app.handle_key_event(KeyCode::Char('d').into());

        app.render(buf.area, &mut buf);

        assert_snapshot!(buf_string(&buf));
    }

    #[test]
    fn test_cancel_rect() {
        let mut app = App::default();
        let mut buf = Buffer::empty(layout::Rect::new(0, 0, 32, 8));

        // Draw one rect and cancel it
        app.handle_key_event(KeyCode::Char('r').into());
        app.handle_key_event(KeyCode::Char('s').into());
        app.handle_key_event(KeyCode::Char('d').into());
        app.handle_key_event(KeyCode::Esc.into());

        // Start drawing another rect
        app.handle_key_event(KeyCode::Char('d').into());
        app.handle_key_event(KeyCode::Char('d').into());
        app.handle_key_event(KeyCode::Char('r').into());
        app.handle_key_event(KeyCode::Char('s').into());
        app.handle_key_event(KeyCode::Char('d').into());

        app.render(buf.area, &mut buf);

        assert_snapshot!(buf_string(&buf));
    }

    fn input(app: &mut App, keys: &[char]) {
        for c in keys {
            app.handle_key_event(KeyCode::Char(*c).into());
        }
    }

    #[test]
    fn test_draw_text() {
        let mut app = App::default();
        let mut buf = Buffer::empty(layout::Rect::new(0, 0, 32, 8));

        // Draw some text and confirm it
        input(&mut app, &['s', 'd', 'i', 'f', 'o', 'o', 'x']);
        app.handle_key_event(KeyCode::Backspace.into());
        app.handle_key_event(KeyCode::Enter.into());

        // Draw some more text and cancel it
        input(&mut app, &['s', 'd', 'i', 'f', 'o', 'o', 'x']);
        app.handle_key_event(KeyCode::Esc.into());

        // Draw some unconfirmed text
        input(&mut app, &['i', 'b', 'a', 'r']);

        app.render(buf.area, &mut buf);

        assert_snapshot!(buf_string(&buf));
    }
}
