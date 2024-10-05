use std::collections::HashMap;

use anyhow::Result;
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
enum RectAction {
    Confirm,
    Cancel,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "snake_case")]
enum LineAction {
    Confirm,
    Cancel,
    AddPoint,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "snake_case")]
enum TextAction {
    Confirm,
    Cancel,
    NewLine,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "snake_case")]
enum NormalAction {
    Quit,
    Save,
}

#[derive(Debug, Deserialize)]
#[serde(default)]
pub struct Binds {
    common: HashMap<String, CommonAction>,
    normal: HashMap<String, NormalAction>,
    rect: HashMap<String, RectAction>,
    line: HashMap<String, LineAction>,
    text: HashMap<String, TextAction>,
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

    use super::*;

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
