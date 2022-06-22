use serde_json::Value;
use tui::widgets::TableState;

use crate::{
    filter::Filter,
    keymap::{Event, KeyMode, Keymap},
    utils::{filter, get_email_list, himalaya_command},
};

use self::data::{Msg, Response, ReviewFlags};

#[derive(Debug)]
pub struct App {
    pub emails: Vec<Msg>,
    pub keymap: Keymap,
    pub command_input: String,
    pub filters: Vec<Filter>,
    pub state: AppState,
    pub should_quit: bool,
    pub need_update: bool,
}

impl App {
    pub fn new() -> Self {
        let emails: Vec<Msg> = serde_json::from_str::<Response>(&get_email_list())
            .unwrap()
            .response;

        App {
            emails,
            keymap: Keymap::default_keymap(),
            state: AppState {
                msg_table: TableState::default(),
                content: (String::new(), 0),
                review_flags: ReviewFlags::default(),
            },
            command_input: String::new(),
            filters: Vec::new(),
            should_quit: false,
            need_update: false,
        }
    }

    /// Processing application event.
    pub fn on_event(&mut self, event: Event) {
        match self.keymap.mode {
            // Process keybind on move mode.
            KeyMode::Motion => match event {
                Event::ExitApp => self.should_quit = true,
                Event::SelectNextMsg => self.state.next(self.emails.len()),
                Event::SelectPrevMsg => self.state.previous(self.emails.len()),
                Event::SwitchMode(mode) => self.keymap.switch_to(mode),
                Event::CancelFilter => {
                    self.filters.pop();
                    self.need_update = true; // Update needed
                    self.command_input.clear();
                }
                Event::ReviewMsg => {
                    if let Some(selected) = self.state.msg_table.selected() {
                        let id = self.emails[selected].id;
                        self.filters.push(Filter(format!("read {}", id)));
                        self.need_update = true;
                    }
                }
                _ => {}
            },
            // Process keybind on input mode.
            KeyMode::Insert => match event {
                Event::Quit => {
                    self.command_input.clear();
                    self.keymap.switch_back();
                }
                Event::Submit => {
                    self.filters.push(Filter(self.command_input.clone()));
                    self.keymap.switch_back();
                    self.need_update = true;
                }
                Event::RawInput(c) => self.command_input.push(c),
                Event::Backspace => {
                    self.command_input.pop();
                }
                _ => {}
            },
            // Process keybind on read mode.
            KeyMode::Review => match event {
                Event::Quit => {
                    self.keymap.switch_to(KeyMode::Motion);
                    self.state.content = (String::new(), 0);
                    self.filters.pop();
                    self.need_update = true;
                    self.command_input.clear();
                }
                Event::ScrollUp => {
                    if self.state.content.1 > 0 {
                        self.state.content.1 -= 1;
                    }
                }
                Event::ScrollDown => self.state.content.1 += 1,
                Event::ShowLinks => {
                    self.state.review_flags.show_links = true;
                    self.command_input.clear();
                    self.command_input = "follow ".to_string();
                    self.keymap.switch_to(KeyMode::Insert);
                }
                Event::ShowStats => self.state.review_flags.show_stats ^= true,
                _ => {}
            },
        }
    }

    pub fn on_tick(&mut self) {
        let mut command: Vec<String> = Vec::new();
        if let Some(Filter(filter)) = self.curr_filter() {
            command = filter.split(' ').map(|s| s.to_string()).collect();
        }

        if !command.is_empty() {
            if self.need_update {
                match command[0].to_uppercase().as_str() {
                    "SEARCH" => {
                        let output = filter(&self.emails);
                        self.emails = output;
                        self.state.msg_table = TableState::default();
                    }
                    "READ" => {
                        // commands that request update from himalaya
                        let output = himalaya_command(command.clone());

                        if let Ok(response) = serde_json::from_str::<Value>(&output) {
                            let content = response.get("response").unwrap().as_str().unwrap();
                            self.state.content = (content.to_string(), 0);
                            self.keymap.switch_to(KeyMode::Review);
                        }
                    }
                    _ => {}
                }

                self.need_update = false;
            } else {
                // process command that does not need to update from himalaya
                match command[0].to_uppercase().as_str() {
                    "FOLLOW" => {
                        // open URL if the argument is correct
                        if let Ok(index) = command[1].parse::<u16>() {
                            if index as usize <= self.state.review_flags.links.len() {
                                open::that(&self.state.review_flags.links[(index - 1) as usize])
                                    .unwrap();
                            }
                        }
                        // clear command input
                        self.state.review_flags.show_links = false;
                        self.keymap.switch_to(KeyMode::Review);
                        self.filters.pop();
                        self.command_input.clear();
                    }
                    _ => {}
                }
            }
        } else if self.need_update {
            self.emails = serde_json::from_str::<Response>(&get_email_list())
                .unwrap()
                .response;
            self.state.msg_table = TableState::default();
            self.need_update = false;
        }
    }

    pub fn curr_filter(&self) -> Option<&Filter> {
        self.filters.last()
    }
}

#[derive(Debug)]
pub struct AppState {
    pub msg_table: TableState,
    pub content: (String, u16),
    pub review_flags: ReviewFlags,
}

impl AppState {
    pub fn next(&mut self, size: usize) {
        if size != 0 {
            let i = match self.msg_table.selected() {
                Some(i) => {
                    if i >= size - 1 {
                        0
                    } else {
                        i + 1
                    }
                }
                None => 0,
            };
            self.msg_table.select(Some(i));
        }
    }

    pub fn previous(&mut self, size: usize) {
        if size != 0 {
            let i = match self.msg_table.selected() {
                Some(i) => {
                    if i == 0 {
                        size - 1
                    } else {
                        i - 1
                    }
                }
                None => 0,
            };
            self.msg_table.select(Some(i));
        }
    }
}

pub mod data {
    use core::fmt;
    use std::fmt::Display;

    use serde::Deserialize;

    #[derive(Debug, Deserialize)]
    pub struct Response {
        pub response: Vec<Msg>,
    }

    #[derive(Debug, Deserialize, Clone)]
    pub struct Msg {
        pub id: usize,
        pub flags: Vec<Flag>,
        pub subject: String,
        pub sender: String,
        pub date: String,
    }

    impl Msg {
        pub fn flags_string(&self) -> String {
            let mut flags = String::new();
            flags.push_str(if self.flags.contains(&Flag::Seen) {
                " "
            } else {
                "✷"
            });
            flags.push_str(if self.flags.contains(&Flag::Answered) {
                "↵"
            } else {
                " "
            });
            flags.push_str(if self.flags.contains(&Flag::Flagged) {
                "⚑"
            } else {
                " "
            });
            flags
        }
    }

    #[derive(Debug, Deserialize, PartialEq, Clone)]
    pub enum Flag {
        Seen,
        Answered,
        Flagged,
        Deleted,
        Draft,
        Recent,
        MayCreate,
        Custom(String),
    }

    impl Display for Flag {
        fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
            let flag = match self {
                Flag::Seen | Flag::Recent | Flag::MayCreate => " ",
                Flag::Answered => "↵",
                Flag::Flagged => "⚑",
                Flag::Deleted => "🗑",
                Flag::Draft => "✉",
                Flag::Custom(c) => c,
            };
            write!(f, "{}", flag)
        }
    }

    #[derive(Debug, Default)]
    pub struct ReviewFlags {
        pub show_stats: bool,
        pub show_links: bool,
        pub links: Vec<String>,
    }
}
