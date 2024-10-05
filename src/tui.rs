use anyhow::Result;

use crossterm::event::{self, Event, KeyCode, KeyEvent, KeyEventKind};
use ratatui::{
    buffer::Buffer,
    layout::{Alignment, Rect},
    style::Stylize,
    symbols::border,
    text::{Line, Text},
    widgets::{
        block::{Position, Title},
        Block, Paragraph, Widget,
    },
    DefaultTerminal, Frame,
};

use crate::canvas::Canvas;

#[derive(Default)]
struct App {
    cursor_x: u16,
    cursor_y: u16,
    canvas: Canvas,
    exit: bool,
    rect: Option<crate::Rect>,
}

impl App {
    fn run(&mut self, mut terminal: DefaultTerminal) -> Result<()> {
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

    fn move_cursor(&mut self, x: i16, y: i16) {
        self.cursor_x = self.cursor_x.saturating_add_signed(x);
        self.cursor_y = self.cursor_y.saturating_add_signed(y);
        log::debug!("Moved cursor to ({}, {})", self.cursor_x, self.cursor_y);
        if let Some(rect) = &mut self.rect {
            rect.x2 = self.cursor_x;
            rect.y2 = self.cursor_y;
            log::debug!("Updated rect to {rect:?}");
        }
    }

    fn handle_key_event(&mut self, key_event: KeyEvent) {
        match key_event.code {
            KeyCode::Char('q') => self.exit = true,

            KeyCode::Char('w') => self.move_cursor(0, -1),
            KeyCode::Char('s') => self.move_cursor(0, 1),
            KeyCode::Char('a') => self.move_cursor(-1, 0),
            KeyCode::Char('d') => self.move_cursor(1, 0),

            KeyCode::Char('r') => {
                self.rect = Some(crate::Rect {
                    x1: self.cursor_x,
                    y1: self.cursor_y,
                    x2: 0,
                    y2: 0,
                });
                log::debug!("Added rect {:?}", self.rect);
                self.move_cursor(1, 1);
            }

            _ => {}
        }
    }
}

impl Widget for &App {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let title = Title::from("Clart".bold());
        let instructions = Title::from(Line::from(vec![
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
                    .position(Position::Bottom),
            )
            .border_set(border::THICK);

        // TODO: have separate scratch layer
        let mut canvas = self.canvas.clone();

        if let Some(rect) = &self.rect {
            log::debug!("Drawing rect: {rect:?}");
            rect.draw(&mut canvas);
        }

        let text = Text::raw(canvas.to_string());
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

    #[test]
    fn test_render_empty() {
        let app = App::default();
        let mut buf = Buffer::empty(Rect::new(0, 0, 50, 4));

        app.render(buf.area, &mut buf);

        let actual = buf
            .content
            .chunks(buf.area.width as usize)
            .map(|line| {
                line.iter()
                    .map(|cell| cell.symbol().to_string())
                    .collect::<Vec<_>>()
                    .join("")
            })
            .collect::<Vec<_>>()
            .join("\n");

        // note ratatui also has an assert_buffer_eq! macro that can be used to
        // compare buffers and display the differences in a more readable way
        assert_snapshot!(actual);
    }

    // #[test]
    // fn handle_key_event() {
    //     let mut app = App::default();
    //     app.handle_key_event(KeyCode::Right.into());
    //     assert_eq!(app.counter, 1);

    //     app.handle_key_event(KeyCode::Left.into());
    //     assert_eq!(app.counter, 0);

    //     let mut app = App::default();
    //     app.handle_key_event(KeyCode::Char('q').into());
    //     assert!(app.exit);
    // }
}
