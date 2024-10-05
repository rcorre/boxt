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
    cursor: (u16, u16),
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
        frame.set_cursor_position(self.cursor);
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
        self.cursor.1 = self.cursor.1.saturating_add_signed(y);
        self.cursor.0 = self.cursor.0.saturating_add_signed(x);
        if let Some(rect) = &mut self.rect {
            rect.w = rect.w.saturating_add_signed(x as isize);
            rect.h = rect.h.saturating_add_signed(y as isize);
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
                log::debug!("Adding rect");
                self.rect = Some(crate::Rect {
                    x: self.cursor.0 as usize,
                    y: self.cursor.1 as usize,
                    w: 0,
                    h: 0,
                });
                self.move_cursor(1, 1);
            }

            _ => {}
        }
    }
}

impl Widget for &App {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let title = Title::from(" Counter App Tutorial ".bold());
        let instructions = Title::from(Line::from(vec![
            " Decrement ".into(),
            "<Left>".blue().bold(),
            " Increment ".into(),
            "<Right>".blue().bold(),
            " Quit ".into(),
            "<Q> ".blue().bold(),
        ]));
        let block = Block::new()
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
    use ratatui::style::Style;

    #[test]
    fn render() {
        let app = App::default();
        let mut buf = Buffer::empty(Rect::new(0, 0, 50, 4));

        app.render(buf.area, &mut buf);

        let mut expected = Buffer::with_lines(vec![
            "┏━━━━━━━━━━━━━ Counter App Tutorial ━━━━━━━━━━━━━┓",
            "┃                    Value: 0                    ┃",
            "┃                                                ┃",
            "┗━ Decrement <Left> Increment <Right> Quit <Q> ━━┛",
        ]);
        let title_style = Style::new().bold();
        let counter_style = Style::new().yellow();
        let key_style = Style::new().blue().bold();
        expected.set_style(Rect::new(14, 0, 22, 1), title_style);
        expected.set_style(Rect::new(28, 1, 1, 1), counter_style);
        expected.set_style(Rect::new(13, 3, 6, 1), key_style);
        expected.set_style(Rect::new(30, 3, 7, 1), key_style);
        expected.set_style(Rect::new(43, 3, 4, 1), key_style);

        // note ratatui also has an assert_buffer_eq! macro that can be used to
        // compare buffers and display the differences in a more readable way
        assert_eq!(buf, expected);
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
