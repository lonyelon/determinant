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

use crate::app::{App, AppResult, AppMode};
use crossterm::event::{KeyCode, KeyEvent};
use crate::client::{sync};

/// Handles the key events and updates the state of [`App`].
pub fn handle_key_events(key_event: KeyEvent, app: &mut App) -> AppResult<()> {
    let window_count = app.windows.len();
    let mut window = &mut app.windows[app.selected_window];

    match app.mode {
        AppMode::Normal => match key_event.code {
            KeyCode::Enter => {
                if window.written_msg != "" {
                    app.send_message();
                } else {
                    app.sel_room();
                }
            }

            KeyCode::Up => {
                if window.selected_room > 0 {
                    window.selected_room -= 1;
                }
            }

            KeyCode::Down => {
                if window.selected_room < app.holder.rooms.keys().len() - 1 {
                    window.selected_room += 1;
                }
            }

            KeyCode::Left => {
                if app.selected_window > 0 {
                    app.selected_window -= 1;
                }
            }

            KeyCode::Right => {
                if app.selected_window < window_count - 1 {
                    app.selected_window += 1;
                }
            }

            KeyCode::Char('i') => {
                app.mode = AppMode::Insert;
            }

            KeyCode::Char('v') => {
                app.add_window();
            }

            KeyCode::Char('s') => {
                sync(&mut app.holder);
            }

            KeyCode::Char('q') => {
                if window_count > 1 && window.selected_room_id == "" {
                    app.windows.remove(app.selected_window);
                    if app.selected_window != 0 {
                        app.selected_window -= 1;
                    }
                } else if window.selected_room_id == "" {
                    app.running = false;
                } else {
                    window.written_msg = String::new();
                    window.selected_room_id = String::new();
                }
            }

            _ => {}
        }

        AppMode::Insert => match key_event.code {
            KeyCode::Esc => {
                app.mode = AppMode::Normal;
            }

            KeyCode::Backspace => {
                if window.written_msg.len() > 0 && window.selected_char > 1 {
                    let selected_char = window.selected_char;
                    window.written_msg.remove(selected_char - 2);
                    window.selected_char -= 1;
                }
            }

            KeyCode::Left => {
                if window.selected_char > 0 {
                    window.selected_char -= 1;
                }
            }

            KeyCode::Right => {
                if window.selected_char < window.written_msg.len() {
                    window.selected_char += 1;
                }
            }

            KeyCode::Enter => {
                let selected_char = window.selected_char;
                window.written_msg.insert(selected_char - 1, '\n');
                window.selected_char += 1;
            }

            KeyCode::Char(c) => {
                let pos = window.selected_char;
                window.written_msg.insert(pos - 1, c);
                window.selected_char += 1;
            }

            _ => {}
        }
    }
    Ok(())
}
