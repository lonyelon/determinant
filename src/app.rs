/* 
 * Determinant: a matrix CLI client inspired by VIM.
 * Copyright (C) 2021  Sergio Miguéns Iglesias <sergio@lony.xyz>
 *
 * This program is free software: you can redistribute it and/or modify it under
 * the terms of the GNU General Public License as published by the Free Software
 * Foundation, either version 3 of the License, or (at your option) any later
 * version.
 *
 * This program is distributed in the hope that it will be useful, but WITHOUT
 * ANY WARRANTY; without even the implied warranty of MERCHANTABILITY or FITNESS
 * FOR A PARTICULAR PURPOSE.  See the GNU General Public License for more
 * details.

 * You should have received a copy of the GNU General Public License along with
 * this program.  If not, see <https://www.gnu.org/licenses/>.
 */

use std::error;
use std::collections::HashMap;
use tui:: {
    backend::Backend,
    layout::Alignment,
    style::{Color, Style},
    terminal::Frame,
    widgets::{Block, Borders, Paragraph, List, ListState, ListItem},
};
use crate::client::{DataHolder, Server};

/// Application result type.
pub type AppResult<T> = std::result::Result<T, Box<dyn error::Error>>;

/// Modes the app can be on. Valid modes are:
///     * Normal - the app is taking commands.
///     * Insert - Allows to enter text.
pub enum AppMode {
    Normal,
    Insert,
}

/// Window to show data on screen.
pub struct MessageWindow {
    pub selected_room_id: String,
    pub selected_room: usize,

    pub written_msg: String,
    pub selected_char: usize,
}

/// Application.
pub struct App {
    pub running: bool,
    pub holder: DataHolder,
    pub selected_room: usize,
    pub selected_room_id: String,
    pub mode: AppMode,

    pub left_panel_width: u16,
    pub invites_height: u16,

    pub written_msg: String,
    pub selected_char: usize,

    pub selected_window: usize,
    pub windows: Vec<MessageWindow>,
}

impl Default for App {
    fn default() -> Self {
        Self {
            holder: DataHolder {
                server: Server{
                    address: String::from("YOUR-SERVER"),
                },
                rooms: HashMap::new(),
                users: HashMap::new(),
                room_invites: vec![],
                token: String::new(),
                user_id: String::from("YOUR-USER"),
                next_batch: String::new(),
            },
            running: true,
            selected_room: 0,
            selected_room_id: String::new(),
            mode: AppMode::Normal,
            left_panel_width: 0,
            invites_height: 10,

            written_msg: String::new(),
            selected_char: 0,

            selected_window: 0,
            windows: vec![MessageWindow {
                selected_room: 0,
                selected_room_id: String::new(),
                written_msg: String::new(),
                selected_char: 1,
            }],
        }
    }
}

impl App {
    /// Constructs a new instance of [`App`].
    pub fn new() -> Self {
        Self::default()
    }

    /// Handles the tick event of the terminal.
    pub fn tick(&self) {}

    pub fn add_window(&mut self) {
        let window = MessageWindow {
            selected_room: 0,
            selected_room_id: String::new(),
            written_msg: String::new(),
            selected_char: 1,
        };

        self.windows.push(window);
    }

    pub fn sel_room(&mut self) {
        let mut window = &mut self.windows[self.selected_window];
        let room_list: Vec<&String> = self.holder.rooms.keys().clone()
            .collect();    
        if window.selected_room_id == "" {
            window.selected_room_id = room_list[window.selected_room].clone();
        }
    }

    /// Renders the user interface widgets.
    pub fn render<B: Backend>(&mut self, frame: &mut Frame<'_, B>) {
        let mut room_list: Vec<&String> = self.holder.rooms.keys().clone()
            .collect();
        //room_list.sort();
        self.selected_room_id = room_list[self.selected_room].clone();

        // Messages.
        self._render_messages(frame, &room_list);

        // LOWER BAR
        frame.render_widget(
            Paragraph::new("Logged in as [USERNAME]")
                .block(Block::default().borders(Borders::NONE))
                    .style(Style::default()
                        .fg(Color::Black)
                        .bg(Color::White))
                    .alignment(Alignment::Left),
            tui::layout::Rect {
                x: 0,
                y: frame.size().height - match self.mode {
                        AppMode::Normal => { 1 },
                        _ => { 2 },
                    },
                width: frame.size().width/2,
                height: 1,
            },
        );

        frame.render_widget(
            Paragraph::new("Determinant Alpha 0.1")
                .block(Block::default().borders(Borders::NONE))
                    .style(Style::default()
                        .fg(Color::Black)
                        .bg(Color::White))
                    .alignment(Alignment::Right),
            tui::layout::Rect {
                x: frame.size().width/2,
                y: frame.size().height - match self.mode {
                        AppMode::Normal => { 1 },
                        _ => { 2 },
                    },
                width: frame.size().width/2,
                height: 1,
            },
        );

        match self.mode {
            AppMode::Insert => {
                frame.render_widget(
                    Paragraph::new("-- INSERT --")
                        .block(Block::default().borders(Borders::NONE))
                        .style(Style::default()
                            .fg(Color::Black)
                            .bg(Color::Green))
                        .alignment(Alignment::Left),
                    tui::layout::Rect {
                        x: 0,
                        y: frame.size().height - 1,
                        width: frame.size().width,
                        height: 1,
                    },
                );
            }

            AppMode::Normal => {
            }
        }
    }

    /// Show rooms list.
    fn _render_room_list<B: Backend>(&self, frame: &mut Frame<'_, B>,
        room_list: &Vec<&String>, window_i: i32, window_x: u16, window_w: u16,
        window_borders: Borders) {

        let mut items: Vec<ListItem> = vec![];
        for room in room_list {
            let members = &self.holder.rooms[&room[..]].members;
            if members.len() == 2 {
                if members[0] == self.holder.user_id {
                    items.push(ListItem::new(self.holder.users.get(&members[1])
                        .unwrap().name.clone()));
                } else {
                    items.push(ListItem::new(self.holder.users.get(&members[0])
                        .unwrap().name.clone()));
                }
            } else {
                items.push(ListItem::new(&room[..]));
            }
        }

        // Color for window, if selected it varies.
        let mut window_color = Color::White;
        if window_i as usize == self.selected_window {
            window_color = Color::Red;
        }

        let items = List::new(items).block(Block::default()
            .title("Room list")
            .title_style(Style::default()
                .fg(Color::Black))
            .borders(window_borders)
            .border_style(Style::default()
                .fg(Color::Black)
                .bg(window_color)))
            .style(Style::default()
                .fg(Color::White))
            .highlight_style(Style::default()
                .fg(Color::Black)
                .bg(Color::White))
            .highlight_symbol("");

        let mut state = ListState::default();
        if window_i == -1 {
            state.select(Some(self.selected_room));
        } else {
            state.select(Some(self.windows[window_i as usize].selected_room));
        }
        
        frame.render_stateful_widget(items, tui::layout::Rect {
                x: window_x,
                y: 0,
                width: window_w,
                height: frame.size().height - match self.mode {
                        AppMode::Insert => { 2 },
                        _ => { 1 },
                    },
            }, &mut state);
    }

    /// Render the "messages" windows for the app.
    fn _render_messages<B: Backend>(&self, frame: &mut Frame<'_, B>,
        room_list: &Vec<&String>) {

        for (window_i, window) in self.windows.iter().enumerate() {
            /*
             * Calculate the size and position for the window. If the window
             * sizes don't add to the whole terminal size, some size is added to
             * the first and the rest are pushed to the right to fill the space.
             *
             * TODO Make this prettier.
             * TODO This code is duplicated, make it into a function?
             * TODO Make size factor distributable between windows so the first
             *      one doesn't become bigger than necessary.
             */
            let mut window_w = ((frame.size().width as usize)/
                self.windows.len()) as u16;
            let mut window_x = window_w*(window_i as u16);
            let round_factor = (frame.size().width - window_w*(
                    self.windows.len() as u16)) as u16;
            if window_i == 0 {
                window_w += round_factor;
            } else {
                window_x += round_factor;
            }
            // Selected window draws on top of the others.
            if self.selected_window > 0 {
                if window_i as usize == self.selected_window - 1 {
                    window_w -= 1;
                } else if window_i as usize == self.selected_window {
                    window_x -= 1;
                    window_w += 1;
                }
            }
            
            // Calculate window borders.
            let window_borders = {
                let mut borders = Borders::TOP;
                
                if window_i != 0 && window_i as usize == self.selected_window {
                    borders |= Borders::LEFT;   
                }

                if window_i as usize != self.windows.len() - 1 && window_i
                    as usize + 1 != self.selected_window {
                    
                    borders |= Borders::RIGHT;
                }
                borders
            };

            if window.selected_room_id == "" {
                self._render_room_list(frame, room_list, window_i as i32,
                    window_x, window_w, window_borders);
                continue;
            }

            // Count the ammount of new lines.
            let newline_count: Vec<&str> = window.written_msg.matches("\n")
                .collect();
            let newline_count = newline_count.len();
            if newline_count > std::u16::MAX as usize {
                panic!("Somehow you typed so many new lines in your message the
                    computer could not count them, congrats I guess.");
            }
            let newline_count = newline_count as u16;

            // Genrate the message and sender list.
            let mut msg_list: Vec<ListItem> = vec![];
            let mut sender_list: Vec<ListItem> = vec![];
            let room_data = &self.holder.rooms.get(&window.selected_room_id);
            for msg in &room_data.unwrap().messages {
                msg_list.push(ListItem::new(&msg.content[..]));

                let alias = self.holder.users.get(&msg.sender);
                if alias.is_none() {
                    sender_list.push(ListItem::new(""));
                    continue;
                }
                let alias = alias.unwrap();
                sender_list.push(ListItem::new(&alias.name[..]));
                let newline_count: Vec<&str> = msg.content.matches("\n")
                    .collect();
                for _ in newline_count {
                    sender_list.push(ListItem::new(&alias.name[..]));
                }
            }

            // List for rust-tui to render.
            let msg_borders = Borders::RIGHT;
            let msg_items = List::new(msg_list).block(Block::default())
                .style(Style::default()
                    .fg(Color::White));

            // List for rust-tui to render.
            let sender_items = List::new(sender_list).block(Block::default()
                .title_style(Style::default()
                    .fg(Color::Black)
                    .bg(Color::White))
                .borders(Borders::RIGHT))
                .style(Style::default()
                    .fg(Color::White));

            // Temp variable so rustc is happy.
            let mut state = ListState::default();

            // Color for window, if selected it varies.
            let mut window_color = Color::White;
            if window_i == self.selected_window {
                window_color = Color::Red;
            }

            let lowbar = match self.mode {
                AppMode::Normal => { 0 },
                _ => { 1 },
            };

            // Draw main box for title.
            let room_title = &window.selected_room_id[..];
            let room_title = ["Messages for room ", room_title].join(" ");
            frame.render_widget(Paragraph::new("").block(Block::default()
                    .title(room_title)
                    .borders(window_borders)
                    .border_style(Style::default()
                        .fg(Color::Black)
                        .bg(window_color)))
                    .style(Style::default()
                       .fg(Color::White)
                       .bg(Color::Black))
                    .alignment(Alignment::Left),
                tui::layout::Rect {
                    x: window_x,
                    y: 0 ,
                    width: window_w,
                    height: frame.size().height - 1 - match self.mode {
                            AppMode::Normal => { 0 },
                            _ => { 1 },
                        },
                },
                );

            let lowbar_height = match self.mode {
                AppMode::Normal => { 3 },
                _ => { 4 },
            };

            let selected = window_i as usize == self.selected_window;
            let move_x = if selected && window_i != 0 { 1 } else { 0 };
            let move_w = if selected && window_i as usize != self.windows.len(){
                2
            } else {
                0
            };

            // Draw senders for room.
            frame.render_stateful_widget(sender_items, tui::layout::Rect {
                    x: window_x + move_x,
                    y: 1,
                    width: window_w/5,
                    height: frame.size().height - newline_count - lowbar -
                        lowbar_height,
                }, &mut state);

            // Draw messages for room.
            frame.render_stateful_widget(msg_items, tui::layout::Rect {
                    x: window_x + window_w/5 + move_x,
                    y: 1,
                    width: window_w - window_w/5 - move_w,
                    height: frame.size().height - newline_count - lowbar -
                        lowbar_height,
                }, &mut state);

            /* 
             * Draw input bar if necessary. This bar is only drawn if INSERT
             * mode is active or if text was previously written to it.
             */
            let in_insert_mode = matches!(&self.mode, AppMode::Insert);
            let is_selected = window_i == self.selected_window;
            if window.written_msg != "" || (in_insert_mode && is_selected) {
                frame.render_widget(
                    Paragraph::new([" $> ", &window.written_msg[..]].join(""))
                        .block(Block::default()
                            .borders(Borders::TOP)
                            .border_style(Style::default()
                                .fg(Color::Black)
                                .bg(Color::White))
                            .style(Style::default()
                                .fg(Color::White)
                                .bg(Color::Black)))
                            .alignment(Alignment::Left),
                    tui::layout::Rect {
                        x: window_x,
                        y: frame.size().height - lowbar - newline_count - 3,
                        width: window_w - 1,
                        height: newline_count + 2});
            }
        }
    }

    /// Sends the written message to the currently selected room.
    pub fn send_message(&mut self) -> json::JsonValue {
        let post_data = json::object! {
            "msgtype": "m.text",
            "body": &self.windows[self.selected_window].written_msg[..],
        };

        self.windows[self.selected_window].written_msg = String::new();
        self.windows[self.selected_window].selected_char = 1;

        let room = self.windows[self.selected_window].selected_room_id.clone();
        self.holder.server.post_data_token(
            &["rooms", &room[..], "send/m.room.message"]
                .join("/")[..],
            &post_data.to_string()[..],
            &self.holder.token[..])
    }
}
