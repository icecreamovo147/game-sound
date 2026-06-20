//! System-wide hotkey registration. Platform permission failures are surfaced as ordinary errors.
use anyhow::{bail, Context, Result};
use global_hotkey::{
    hotkey::{Code, HotKey, Modifiers},
    GlobalHotKeyManager,
};
use std::collections::HashMap;

pub struct HotkeyRegistry {
    manager: GlobalHotKeyManager,
    bindings: HashMap<u32, String>,
}
impl HotkeyRegistry {
    pub fn new() -> Result<Self> {
        Ok(Self {
            manager: GlobalHotKeyManager::new()
                .context("cannot initialise global hotkey manager")?,
            bindings: HashMap::new(),
        })
    }
    pub fn register(&mut self, shortcut: &str) -> Result<u32> {
        let hotkey = parse(shortcut)?;
        if self
            .bindings
            .values()
            .any(|v| v.eq_ignore_ascii_case(shortcut))
        {
            bail!("hotkey conflict: {shortcut}");
        }
        self.manager
            .register(hotkey)
            .with_context(|| format!("cannot register global hotkey {shortcut}"))?;
        self.bindings
            .insert(hotkey.id(), shortcut.to_ascii_lowercase());
        Ok(hotkey.id())
    }
    pub fn unregister_all(&mut self) -> Result<()> {
        let keys = self
            .bindings
            .values()
            .map(|shortcut| parse(shortcut))
            .collect::<Result<Vec<_>>>()?;
        self.manager.unregister_all(&keys)?;
        self.bindings.clear();
        Ok(())
    }
    pub fn action_for(&self, id: u32) -> Option<&str> {
        self.bindings.get(&id).map(String::as_str)
    }
    pub fn shortcuts(&self) -> Vec<String> {
        self.bindings.values().cloned().collect()
    }
}
pub fn parse(shortcut: &str) -> Result<HotKey> {
    let normalized = shortcut.trim().to_lowercase();
    let mut parts = normalized
        .split('+')
        .map(str::trim)
        .filter(|s| !s.is_empty())
        .collect::<Vec<_>>();
    let key = parts.pop().context("a hotkey needs a key")?;
    let mut mods = Modifiers::empty();
    for modifier in parts {
        match modifier {
            "ctrl" | "control" => mods |= Modifiers::CONTROL,
            "alt" | "option" => mods |= Modifiers::ALT,
            "shift" => mods |= Modifiers::SHIFT,
            "meta" | "cmd" | "command" | "win" => mods |= Modifiers::META,
            _ => bail!("unknown modifier: {modifier}"),
        }
    }
    let code = match key {
        "up" => Code::ArrowUp,
        "down" => Code::ArrowDown,
        "left" => Code::ArrowLeft,
        "right" => Code::ArrowRight,
        "space" => Code::Space,
        "f1" => Code::F1,
        "f2" => Code::F2,
        "f3" => Code::F3,
        "f4" => Code::F4,
        "f5" => Code::F5,
        "f6" => Code::F6,
        "f7" => Code::F7,
        "f8" => Code::F8,
        "f9" => Code::F9,
        "f10" => Code::F10,
        "f11" => Code::F11,
        "f12" => Code::F12,
        x if x.len() == 1 && x.as_bytes()[0].is_ascii_alphanumeric() => match x {
            "0" => Code::Digit0,
            "1" => Code::Digit1,
            "2" => Code::Digit2,
            "3" => Code::Digit3,
            "4" => Code::Digit4,
            "5" => Code::Digit5,
            "6" => Code::Digit6,
            "7" => Code::Digit7,
            "8" => Code::Digit8,
            "9" => Code::Digit9,
            "a" => Code::KeyA,
            "b" => Code::KeyB,
            "c" => Code::KeyC,
            "d" => Code::KeyD,
            "e" => Code::KeyE,
            "f" => Code::KeyF,
            "g" => Code::KeyG,
            "h" => Code::KeyH,
            "i" => Code::KeyI,
            "j" => Code::KeyJ,
            "k" => Code::KeyK,
            "l" => Code::KeyL,
            "m" => Code::KeyM,
            "n" => Code::KeyN,
            "o" => Code::KeyO,
            "p" => Code::KeyP,
            "q" => Code::KeyQ,
            "r" => Code::KeyR,
            "s" => Code::KeyS,
            "t" => Code::KeyT,
            "u" => Code::KeyU,
            "v" => Code::KeyV,
            "w" => Code::KeyW,
            "x" => Code::KeyX,
            "y" => Code::KeyY,
            "z" => Code::KeyZ,
            _ => unreachable!(),
        },
        _ => bail!("unsupported hotkey key: {key}"),
    };
    Ok(HotKey::new(Some(mods), code))
}

/// Bare keys are reserved for TUI navigation/actions. Modifier combinations
/// remain available because they are unambiguous outside the terminal UI.
pub fn is_reserved_tui_hotkey(shortcut: &str) -> bool {
    let key = shortcut.trim().to_ascii_lowercase();
    if key.contains('+') {
        return false;
    }
    matches!(
        key.as_str(),
        "q" | "?"
            | "tab"
            | "esc"
            | "enter"
            | "space"
            | "c"
            | "s"
            | "/"
            | "r"
            | "a"
            | "e"
            | "d"
            | "b"
            | "n"
            | "m"
            | "o"
            | "l"
            | "t"
            | "p"
            | "f"
            | "g"
            | "i"
            | "k"
            | "v"
            | "x"
            | "h"
            | "u"
            | "y"
            | "z"
            | "+"
            | "-"
            | "1"
            | "2"
            | "3"
            | "["
            | "]"
            | "up"
            | "down"
            | "left"
            | "right"
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn parses_modifiers() {
        assert!(parse("ctrl+alt+1").is_ok());
        assert!(parse("ctrl+what+1").is_err());
    }
    #[test]
    fn reserves_bare_tui_keys_but_allows_modified_keys() {
        assert!(is_reserved_tui_hotkey("b"));
        assert!(is_reserved_tui_hotkey("space"));
        assert!(!is_reserved_tui_hotkey("ctrl+b"));
        assert!(!is_reserved_tui_hotkey("command+1"));
    }
}
