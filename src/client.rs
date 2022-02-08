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

use std:: {
    str,
    collections::HashMap,
    io::Read,
};
use json::JsonValue;
use curl::easy::Easy;

/// Holds the global data for the client.
pub struct DataHolder {
    pub server: Server,
    pub user_id: String,
    pub token: String,

    pub rooms: HashMap<String, RoomData>,
    pub users: HashMap<String, UserData>,
    pub room_invites: Vec<String>,

    pub next_batch: String,
}

pub struct Message {
    pub sender: String,
    pub room: String,
    pub content: String,
    pub msgtype: String
}

pub struct UserData {
    pub name: String,
    pub is_online: bool,
    pub rooms: Vec<String>,
}

pub struct RoomData {
    pub alias: String,
    pub members: Vec<String>,
    pub messages: Vec<Message>,
    pub unread_msgs: u32,
}

pub struct Server {
    pub address: String,
}

impl Server {
    pub fn get_data_token(&self, url: &str, params: Vec<&str>, token: &str)
        -> JsonValue {
        
        let mut params_str = String::from("");
        for (_, param) in params.iter().enumerate() {
            params_str.push_str("&");
            params_str.push_str(param);
        }
        let url = [&self.address[..], "/_matrix/client/r0/", url,
                   "?access_token=", token, &params_str[..]].join("");
        self._perform_request(&url[..], "")
    }

    /// Posts data to the server with a user token.
    pub fn post_data_token(&self, url: &str, data: &str, token: &str)
        -> JsonValue {
        let url = [url, "?access_token=", token].join("");
        self.post_data(&url[..], data)
    }

    /// Posts data to the server without including the user token.
    pub fn post_data(&self, url: &str, data: &str) -> JsonValue {
        let url = &str::replace(url, ":", "%3A")[..];
        let url = [&self.address[..], "/_matrix/client/r0/", url].join("");
        self._perform_request(&url[..], data)
    }

    /// Sends a request to the server.
    fn _perform_request(&self, url: &str, data: &str) -> JsonValue {
        let post_params = data.clone();
        let mut data = data.as_bytes();
        let mut return_data = Vec::new();

        {
            let mut handle = Easy::new();
            handle.url(&url[..]).unwrap();

            // No POST data, don't post!
            if data.len() != 0 {
                handle.post(true).unwrap();
                handle.post_field_size(data.len() as u64).unwrap();
            }

            let mut transfer = handle.transfer();
            
            /* 
             * FIXME This looks ugly, but I can't figure how to merge this with
             * the other "if" statement without rustc complaining.
             */
            if data.len() != 0 {
                transfer.read_function(|buf| {
                    Ok(data.read(buf).unwrap_or(0))
                }).unwrap();
            }

            transfer.write_function(|new_data| {
                return_data.extend_from_slice(new_data);
                Ok(new_data.len())
            }).unwrap();
            transfer.perform().unwrap();
        }

        let return_data = match str::from_utf8(&return_data) {
            Ok(v) => v,
            Err(_) => panic!("Data can not be converted to aray!")
        };

        json::parse(return_data).unwrap()
    }
}

pub fn sync(holder: &mut DataHolder) -> json::JsonValue {
    let mut params = vec![
        //"filter=\"\"",
        //"since=\"\"",
        "full_state=true",
        "set_presence=online",
        //"timeout=100",
    ];

    let token = &holder.token.clone()[..];

    let since = ["since=", &holder.next_batch[..]].join("");
    if holder.next_batch != "" {
        params.push(&since[..]);
    }

    let res = holder.server.get_data_token("sync", params, token);
    // Get joined rooms.
    for (room_id, room) in res["rooms"]["join"].entries() {
        let room_exists = !holder.rooms.contains_key(&room_id.to_string());

        let mut new_room = RoomData {
            alias: String::new(),
            members: vec![],
            messages: vec![],
            unread_msgs: room["unread_notifications"]["notification_count"]
                .as_u32().unwrap()
        };

        // Get messages.
        for event in room["timeline"]["events"].members() {
            if event["type"] == "m.room.message"
                && event["content"]["msgtype"] == "m.text" {
                let msg = Message {
                    content: event["content"]["body"].to_string(),
                    msgtype: "text".to_string(),
                    sender: event["sender"].to_string(),
                    room: room_id.to_string()
                };

                if room_exists {
                    new_room.messages.push(msg);
                } else {
                    let previous_room = holder.rooms
                        .get_mut(&room_id.to_string()).unwrap();
                    previous_room.messages.push(msg);
                }
            }
        }

        // Get state.
        for event in room["state"]["events"].members() {
            if event["type"] == "m.room.member" &&
                event["content"]["membership"] == "join" {

                // Add the room to the user's list.
                let user_id = event["sender"].to_string().clone();
                let user_data = holder.users.get(&user_id[..]);
                if user_data.is_none() {
                    let alias = event["content"]["displayname"]
                        .to_string().clone();

                    let new_user = UserData {
                        name: alias,
                        rooms: vec![room_id.to_string().clone()],
                        is_online: false,
                    };

                    holder.users.insert(user_id.clone(), new_user);
                } else if !holder.users.get(&user_id[..]).unwrap()
                    .rooms.contains(&room_id.to_string()) {
                    holder.users.get_mut(&user_id[..]).unwrap().rooms
                        .push(room_id.to_string().clone());
                }

                new_room.members.push(user_id);
            }
        }

        holder.rooms.entry(room_id.to_string()).or_insert(new_room);
    }

    // Get invites.
    for (room_id, _) in res["rooms"]["invite"].entries() {
        holder.room_invites.push(room_id.to_string())
    }

    holder.next_batch = res["next_batch"].to_string();

    res
}

// Logins in a server given a user nama and password pair.
pub fn login(srv: &Server, user: &str, pass: &str) -> json::JsonValue {
    let login_request = json::object!{
        "type": "m.login.password",
        "identifier": {
            "type": "m.id.user",
            "user": user
        },
        "password": pass
    };

    srv.post_data("login", &json::stringify(login_request)[..])
}
