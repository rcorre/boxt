use std::collections::HashMap;

use anyhow::Result;
use serde::Deserialize;

#[derive(Debug, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum EnterMode {
    Rect,
    Line,
    Text,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Action {
    Quit,
    Save,
    MoveCursor { x: i16, y: i16 },
    Confirm,
    Cancel,
    LineAddPoint,
    TextAddLine,
    EnterMode(EnterMode),
    Delete,
}

#[derive(Debug, Deserialize)]
pub struct BindConfig(pub HashMap<String, Action>);

impl<U> From<U> for BindConfig
where
    U: Into<HashMap<String, Action>>,
{
    fn from(value: U) -> Self {
        Self(value.into())
    }
}

impl std::ops::Index<&str> for BindConfig {
    type Output = Action;

    fn index(&self, index: &str) -> &Self::Output {
        &self.0[index]
    }
}

impl Default for BindConfig {
    fn default() -> Self {
        Self(
            [
                // cursor
                ("w".to_string(), Action::MoveCursor { x: 0, y: -1 }),
                ("a".to_string(), Action::MoveCursor { x: -1, y: 0 }),
                ("s".to_string(), Action::MoveCursor { x: 0, y: 1 }),
                ("d".to_string(), Action::MoveCursor { x: 1, y: 0 }),
                ("S-w".to_string(), Action::MoveCursor { x: 0, y: -4 }),
                ("S-a".to_string(), Action::MoveCursor { x: -4, y: 0 }),
                ("S-s".to_string(), Action::MoveCursor { x: 0, y: 4 }),
                ("S-d".to_string(), Action::MoveCursor { x: 4, y: 0 }),
                // mode
                ("r".to_string(), Action::EnterMode(EnterMode::Rect)),
                ("i".to_string(), Action::EnterMode(EnterMode::Text)),
                ("l".to_string(), Action::EnterMode(EnterMode::Line)),
                // line
                (" ".to_string(), Action::LineAddPoint),
                // general
                ("x".to_string(), Action::Delete),
                ("C-s".to_string(), Action::Save),
                ("q".to_string(), Action::Quit),
                ("enter".to_string(), Action::Confirm),
                ("esc".to_string(), Action::Cancel),
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
    use std::assert_matches::assert_matches;

    #[test]
    fn test_config_binds() {
        let s = toml::toml! {
            [binds]
            w = { "move_cursor" = { x = 0, y = -1 } }
            C-c = "quit"
            s = "save"
        }
        .to_string();

        let c = Config::read(&s).unwrap();
        let b = c.binds;

        assert_matches!(b.0["w"], Action::MoveCursor { x: 0, y: -1 });
        assert_matches!(b.0["C-c"], Action::Quit);
        assert_matches!(b.0["s"], Action::Save);
    }
}
