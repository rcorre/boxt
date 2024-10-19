use std::collections::HashMap;

use anyhow::Result;
use serde::Deserialize;

#[derive(Clone, Debug, Deserialize)]
#[cfg_attr(test, derive(PartialEq))]
#[serde(rename_all = "snake_case")]
pub enum Action {
    Quit,
    Save,

    MoveCursorUp,
    MoveCursorDown,
    MoveCursorLeft,
    MoveCursorRight,

    DrawRect,
    DrawLine,
    DrawText,
    ExitMode,

    LineAddPoint,
    LineMirror,
    TextAddLine,

    Delete,
    Undo,
    Redo,

    SelectRect,
}

#[derive(Debug, Deserialize)]
#[cfg_attr(test, derive(PartialEq, Clone))]
#[serde(untagged)]
pub enum Binding {
    Single(Action),
    Multi(Vec<Action>),
}

#[derive(Debug, Deserialize)]
pub struct BindConfig(pub HashMap<String, Binding>);

impl std::ops::Index<&str> for BindConfig {
    type Output = Binding;

    fn index(&self, index: &str) -> &Self::Output {
        &self.0[index]
    }
}

impl Default for BindConfig {
    fn default() -> Self {
        Self(
            [
                // cursor
                ("w".to_string(), Binding::Single(Action::MoveCursorUp)),
                ("a".to_string(), Binding::Single(Action::MoveCursorLeft)),
                ("s".to_string(), Binding::Single(Action::MoveCursorDown)),
                ("d".to_string(), Binding::Single(Action::MoveCursorRight)),
                (
                    "S-w".to_string(),
                    Binding::Multi(vec![Action::MoveCursorUp; 4]),
                ),
                (
                    "S-a".to_string(),
                    Binding::Multi(vec![Action::MoveCursorLeft; 4]),
                ),
                (
                    "S-s".to_string(),
                    Binding::Multi(vec![Action::MoveCursorDown; 4]),
                ),
                (
                    "S-d".to_string(),
                    Binding::Multi(vec![Action::MoveCursorRight; 4]),
                ),
                // mode
                ("r".to_string(), Binding::Single(Action::DrawRect)),
                ("i".to_string(), Binding::Single(Action::DrawText)),
                ("l".to_string(), Binding::Single(Action::DrawLine)),
                // line
                (" ".to_string(), Binding::Single(Action::LineAddPoint)),
                ("m".to_string(), Binding::Single(Action::LineMirror)),
                // general
                ("x".to_string(), Binding::Single(Action::Delete)),
                ("C-s".to_string(), Binding::Single(Action::Save)),
                ("q".to_string(), Binding::Single(Action::Quit)),
                ("esc".to_string(), Binding::Single(Action::ExitMode)),
                ("enter".to_string(), Binding::Single(Action::ExitMode)),
                ("u".to_string(), Binding::Single(Action::Undo)),
                ("U".to_string(), Binding::Single(Action::Redo)),
                ("m".to_string(), Binding::Single(Action::SelectRect)),
            ]
            .into(),
        )
    }
}

#[derive(Debug, Default, Deserialize)]
#[serde(default)]
pub struct Config {
    pub binds: BindConfig,
}

impl Config {
    pub fn read(s: &str) -> Result<Config> {
        let c: Self = toml::from_str(s)?;
        Ok(c)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_config_binds() {
        let s = toml::toml! {
            [binds]
            w = "move_cursor_down"
            C-c = ["save", "quit"]
            s = "save"
        }
        .to_string();

        let c = Config::read(&s).unwrap();
        let b = c.binds;

        assert_eq!(b.0["w"], Binding::Single(Action::MoveCursorDown));
        assert_eq!(b.0["C-c"], Binding::Multi(vec![Action::Save, Action::Quit]));
        assert_eq!(b.0["s"], Binding::Single(Action::Save));
    }
}
