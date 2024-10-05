use std::collections::HashMap;

use anyhow::Result;
use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use serde::Deserialize;

#[derive(Debug, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum EnterModeAction {
    Normal,
    Rect,
    Line,
    Text,
}

#[derive(Debug, Deserialize)]
#[serde(tag = "command")]
#[serde(rename_all = "snake_case")]
pub enum CommonAction {
    MoveCursor { x: i16, y: i16 },
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum RectAction {
    Confirm,
    Cancel,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum LineAction {
    Confirm,
    Cancel,
    AddPoint,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum TextAction {
    Confirm,
    Cancel,
    NewLine,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum NormalAction {
    Quit,
    Save,
}

#[derive(Debug, Deserialize)]
pub struct BindMap<T>(HashMap<String, T>);

impl<T, U> From<U> for BindMap<T>
where
    U: Into<HashMap<String, T>>,
{
    fn from(value: U) -> Self {
        Self(value.into())
    }
}

impl<T> std::ops::Index<&str> for BindMap<T> {
    type Output = T;

    fn index(&self, index: &str) -> &Self::Output {
        &self.0[index]
    }
}

impl<T> BindMap<T> {
    pub fn get(&self, ev: &KeyEvent) -> Option<&T> {
        match ev.code {
            KeyCode::Backspace => self.0.get("backspace"),
            KeyCode::Enter => self.0.get("enter"),
            KeyCode::Left => self.0.get("left"),
            KeyCode::Right => self.0.get("right"),
            KeyCode::Up => self.0.get("up"),
            KeyCode::Down => self.0.get("down"),
            KeyCode::Home => self.0.get("home"),
            KeyCode::End => self.0.get("end"),
            KeyCode::PageUp => self.0.get("pageup"),
            KeyCode::PageDown => self.0.get("pagedown"),
            KeyCode::Tab => self.0.get("tab"),
            KeyCode::BackTab => self.0.get("backtab"),
            KeyCode::Delete => self.0.get("delete"),
            KeyCode::Insert => self.0.get("insert"),
            KeyCode::F(n) => self.0.get(&format!("f{n}")),
            KeyCode::Char(c) if ev.modifiers.contains(KeyModifiers::SHIFT) => {
                self.0.get(&c.to_uppercase().to_string())
            }
            KeyCode::Char(c) => self.0.get(&c.to_string()),
            KeyCode::Null => self.0.get("null"),
            KeyCode::Esc => self.0.get("esc"),
            KeyCode::CapsLock => self.0.get("capslock"),
            KeyCode::ScrollLock => self.0.get("scrolllock"),
            KeyCode::NumLock => self.0.get("numlock"),
            KeyCode::PrintScreen => self.0.get("printscreen"),
            KeyCode::Pause => self.0.get("pause"),
            KeyCode::Menu => self.0.get("menu"),
            KeyCode::KeypadBegin => self.0.get("keypadbegin"),
            KeyCode::Media(_) => {
                log::warn!("Media keys not supported");
                None
            }
            KeyCode::Modifier(_) => {
                log::warn!("Bare modifiers not supported");
                None
            }
        }
    }
}

#[derive(Debug, Deserialize)]
#[serde(default)]
pub struct Binds {
    common: BindMap<CommonAction>,
    normal: BindMap<NormalAction>,
    rect: BindMap<RectAction>,
    line: BindMap<LineAction>,
    text: BindMap<TextAction>,
}

impl Default for Binds {
    fn default() -> Self {
        Self {
            common: [
                ("w".to_string(), CommonAction::MoveCursor { x: 0, y: -1 }),
                ("a".to_string(), CommonAction::MoveCursor { x: -1, y: 0 }),
                ("s".to_string(), CommonAction::MoveCursor { x: 0, y: 1 }),
                ("d".to_string(), CommonAction::MoveCursor { x: 1, y: 0 }),
            ]
            .into(),
            normal: [
                ("s".to_string(), NormalAction::Save),
                ("q".to_string(), NormalAction::Quit),
            ]
            .into(),
            rect: [
                ("enter".to_string(), RectAction::Confirm),
                ("esc".to_string(), RectAction::Cancel),
            ]
            .into(),
            line: [
                ("enter".to_string(), LineAction::Confirm),
                ("esc".to_string(), LineAction::Cancel),
            ]
            .into(),
            text: [
                ("enter".to_string(), TextAction::Confirm),
                ("esc".to_string(), TextAction::Cancel),
            ]
            .into(),
        }
    }
}

#[derive(Debug, Default, Deserialize)]
#[serde(default)]
pub struct Config {
    pub binds: Binds,
}

impl Config {
    pub fn read(s: &str) -> Result<Config> {
        let c: Self = toml::from_str(s)?;
        Ok(c)
    }
}

#[cfg(test)]
mod tests {
    use std::assert_matches::assert_matches;

    use crossterm::event::KeyModifiers;

    use super::*;

    #[test]
    fn test_bind_get() {
        let b = BindMap::<u32>(
            [
                ("a".to_string(), 0),
                ("b".to_string(), 1),
                ("esc".to_string(), 2),
                ("enter".to_string(), 3),
                ("f5".to_string(), 4),
                ("B".to_string(), 5),
            ]
            .into(),
        );

        assert_eq!(
            b.get(&KeyEvent::new(KeyCode::Char('a'), KeyModifiers::empty())),
            Some(&0)
        );
        assert_eq!(
            b.get(&KeyEvent::new(KeyCode::Char('b'), KeyModifiers::empty())),
            Some(&1)
        );
        assert_eq!(
            b.get(&KeyEvent::new(KeyCode::Esc, KeyModifiers::empty())),
            Some(&2)
        );
        assert_eq!(
            b.get(&KeyEvent::new(KeyCode::Enter, KeyModifiers::empty())),
            Some(&3)
        );
        assert_eq!(
            b.get(&KeyEvent::new(KeyCode::Char('B'), KeyModifiers::empty())),
            Some(&5)
        );
        assert_eq!(
            b.get(&KeyEvent::new(KeyCode::Char('b'), KeyModifiers::SHIFT)),
            Some(&5)
        );
        assert_eq!(
            b.get(&KeyEvent::new(KeyCode::Char('B'), KeyModifiers::SHIFT)),
            Some(&5)
        );
    }

    #[test]
    fn test_config_binds() {
        let s = toml::toml! {
            [binds.common]
            w = { "command" = "move_cursor", x = 0, y = -1 }
            [binds.normal]
            q = "quit"
            s = "save"
        }
        .to_string();

        let c = Config::read(&s).unwrap();
        assert_matches!(
            c.binds.common["w"],
            CommonAction::MoveCursor { x: 0, y: -1 }
        );
        assert_matches!(c.binds.normal["q"], NormalAction::Quit);
        assert_matches!(c.binds.normal["s"], NormalAction::Save);

        // default
        assert_matches!(c.binds.rect["enter"], RectAction::Confirm);
    }
}
