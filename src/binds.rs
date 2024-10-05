use std::collections::HashMap;

use anyhow::{bail, Result};
use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};

use crate::config::{self, Action};

#[derive(Hash, PartialEq, Eq)]
struct Key {
    code: KeyCode,
    modifiers: KeyModifiers,
}

#[derive(Default)]
pub struct Binds(HashMap<Key, Action>);

impl Binds {
    pub fn get(&self, ev: &KeyEvent) -> Option<&Action> {
        let key = Key {
            code: ev.code,
            modifiers: ev.modifiers,
        };
        self.0.get(&key)
    }

    pub fn from_config(c: config::BindConfig) -> Result<Self> {
        let mut m = HashMap::new();
        for (k, v) in c.0.into_iter() {
            m.insert(map_key(&k)?, v);
        }
        Ok(Self(m))
    }
}

fn map_key(key: &str) -> Result<Key> {
    let mut parts = key.split('-').rev();
    let Some(code) = parts.next() else {
        bail!("Empty key");
    };
    let code = match code {
        c if c.len() == 1 => KeyCode::Char(c.chars().next().unwrap()),
        s if s.starts_with("f") => {
            let (_, num) = s.split_at(1);
            let num = num.parse()?;
            KeyCode::F(num)
        }
        "backspace" => KeyCode::Backspace,
        "enter" => KeyCode::Enter,
        "left" => KeyCode::Left,
        "right" => KeyCode::Right,
        "up" => KeyCode::Up,
        "down" => KeyCode::Down,
        "home" => KeyCode::Home,
        "end" => KeyCode::End,
        "pageup" => KeyCode::PageUp,
        "pagedown" => KeyCode::PageDown,
        "tab" => KeyCode::Tab,
        "backtab" => KeyCode::BackTab,
        "delete" => KeyCode::Delete,
        "insert" => KeyCode::Insert,
        "null" => KeyCode::Null,
        "esc" => KeyCode::Esc,
        "capslock" => KeyCode::CapsLock,
        "scrolllock" => KeyCode::ScrollLock,
        "numlock" => KeyCode::NumLock,
        "print" => KeyCode::PrintScreen,
        "pause" => KeyCode::Pause,
        "menu" => KeyCode::Menu,
        "keypadbegin" => KeyCode::KeypadBegin,
        unknown => bail!("Unknown key: {unknown}"),
    };
    let mut modifiers = KeyModifiers::empty();
    for p in parts {
        modifiers.insert(match p {
            "s" | "S" => KeyModifiers::SHIFT,
            "c" | "C" => KeyModifiers::CONTROL,
            "a" | "A" => KeyModifiers::ALT,
            m => bail!(format!("Unknown modifier {m}")),
        });
    }
    Ok(Key { code, modifiers })
}

impl Binds {}

#[cfg(test)]
mod tests {
    use std::assert_matches::assert_matches;

    use config::BindConfig;
    use crossterm::event::KeyModifiers;

    use super::*;

    #[test]
    fn test_binds() {
        let b = Binds::from_config(BindConfig(
            [
                ("s".into(), Action::MoveCursor { x: -1, y: 0 }),
                ("C-s".into(), Action::Save),
                ("enter".into(), Action::Confirm),
                ("C-S-tab".into(), Action::LineAddPoint),
                ("a-enter".into(), Action::TextAddLine),
            ]
            .into(),
        ))
        .unwrap();

        assert_matches!(
            b.get(&KeyEvent::new(KeyCode::Char('s'), KeyModifiers::empty())),
            Some(Action::MoveCursor { x: -1, y: 0 })
        );
        assert_matches!(
            b.get(&KeyEvent::new(KeyCode::Char('s'), KeyModifiers::CONTROL)),
            Some(Action::Save)
        );
        assert_matches!(
            b.get(&KeyEvent::new(KeyCode::Char('s'), KeyModifiers::ALT)),
            None
        );
        assert_matches!(
            b.get(&KeyEvent::new(KeyCode::Enter, KeyModifiers::empty())),
            Some(Action::Confirm)
        );
        assert_matches!(
            b.get(&KeyEvent::new(
                KeyCode::Tab,
                KeyModifiers::SHIFT | KeyModifiers::CONTROL
            )),
            Some(Action::LineAddPoint)
        );
        assert_matches!(
            b.get(&KeyEvent::new(KeyCode::Enter, KeyModifiers::ALT,)),
            Some(Action::TextAddLine)
        );
    }
}
