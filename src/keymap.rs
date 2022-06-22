use core::fmt;
use std::collections::HashMap;

use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};

#[derive(Debug)]
pub struct Keymap {
    pub mode: KeyMode,
    pub prev_mode: KeyMode,
    keybinds: HashMap<KeyMode, Vec<Keybind>>,
}

impl Keymap {
    pub fn default_keymap() -> Self {
        Keymap {
            mode: KeyMode::Motion,
            prev_mode: KeyMode::Motion,
            keybinds: HashMap::from([
                (
                    KeyMode::Motion,
                    vec![
                        Keybind(KeyCode::Char('j'), KeyModifiers::NONE, Event::SelectNextMsg),
                        Keybind(KeyCode::Char('k'), KeyModifiers::NONE, Event::SelectPrevMsg),
                        Keybind(KeyCode::Esc, KeyModifiers::NONE, Event::ExitApp),
                        Keybind(
                            KeyCode::Char(':'),
                            KeyModifiers::NONE,
                            Event::SwitchMode(KeyMode::Insert),
                        ),
                        Keybind(KeyCode::Char('q'), KeyModifiers::NONE, Event::CancelFilter),
                        Keybind(KeyCode::Enter, KeyModifiers::NONE, Event::ReviewMsg),
                    ],
                ),
                (
                    KeyMode::Insert,
                    vec![
                        Keybind(KeyCode::Char('d'), KeyModifiers::CONTROL, Event::Quit),
                        Keybind(KeyCode::Esc, KeyModifiers::NONE, Event::Quit),
                        Keybind(KeyCode::Enter, KeyModifiers::NONE, Event::Submit),
                        Keybind(KeyCode::Backspace, KeyModifiers::NONE, Event::Backspace),
                    ],
                ),
                (
                    KeyMode::Review,
                    vec![
                        Keybind(KeyCode::Char('q'), KeyModifiers::NONE, Event::Quit),
                        Keybind(KeyCode::Char('j'), KeyModifiers::NONE, Event::ScrollDown),
                        Keybind(KeyCode::Char('k'), KeyModifiers::NONE, Event::ScrollUp),
                        Keybind(KeyCode::Char('f'), KeyModifiers::NONE, Event::ShowLinks),
                        Keybind(KeyCode::Char('s'), KeyModifiers::NONE, Event::ShowStats),
                    ],
                ),
            ]),
        }
    }

    pub fn on_key(&self, key: KeyEvent) -> Vec<Event> {
        let mut events = Vec::new();

        for keybind in self.keybinds.get(&self.mode).unwrap() {
            if let Some(event) = keybind.match_key(key.code, key.modifiers) {
                events.push(event);
            }
        }

        // if input mode is KeyMode::Input, map all chars into RawInput
        if self.mode == KeyMode::Insert {
            if let KeyCode::Char(c) = key.code {
                events.push(Event::RawInput(c));
            }
        }

        events
    }

    pub fn switch_to(&mut self, mode: KeyMode) {
        self.prev_mode = self.mode.clone();
        self.mode = mode;
    }

    pub fn switch_back(&mut self) {
        let temp = self.prev_mode.clone();
        self.prev_mode = self.mode.clone();
        self.mode = temp;
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum KeyMode {
    Motion,
    Insert,
    Review,
}

impl fmt::Display for KeyMode {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{}",
            match *self {
                KeyMode::Insert => "INSERT",
                KeyMode::Motion => "MOTION",
                KeyMode::Review => "REVIEW",
            }
        )
    }
}

#[derive(Debug)]
pub struct Keybind(KeyCode, KeyModifiers, Event);

impl Keybind {
    pub fn match_key(&self, key: KeyCode, modifier: KeyModifiers) -> Option<Event> {
        if self.0 == key && self.1 == modifier {
            Some(self.2.clone())
        } else {
            None
        }
    }
}

#[derive(Debug, Clone)]
pub enum Event {
    ExitApp,
    SelectNextMsg,
    SelectPrevMsg,
    ReviewMsg,
    Quit,
    Submit,
    RawInput(char),
    Backspace,
    CancelFilter,
    SwitchMode(KeyMode),
    ScrollUp,
    ScrollDown,
    ShowLinks,
    ShowStats,
}
