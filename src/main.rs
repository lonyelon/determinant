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

use std::io;
use tui::backend::CrosstermBackend;
use tui::Terminal;
use determinant:: {
    app::{App, AppResult},
    event::{Event, EventHandler},
    handler::handle_key_events,
    tui::Tui,
    client::{login, sync},
};

fn main() -> AppResult<()> {
    
    // Print license as recommended by the FSF.
    println!("Determinant: the CLI matrix client  Copyright (C) 2021 Sergio \
              Miguéns Iglesias\nThis program comes with ABSOLUTELY NO WARRANTY; \
              for details type \"show w\". This is free software, and you are \
              welcome to redistribute it under certain conditions; type \"show \
              c\" for details.");

    // Create an application.
    let mut app = App::new();

    // Initialize the terminal user interface.
    let backend = CrosstermBackend::new(io::stderr());
    let terminal = Terminal::new(backend)?;
    let events = EventHandler::new(250);
    let mut tui = Tui::new(terminal, events);
    tui.init()?;

    // Get user token from server.
    // TODO Store preevious tokens.
    // TODO Ask for credentials.
    let token = login(&app.holder.server, "YOUR-USERNAME", "YOUR-PASSWORD");
    let token = token["access_token"].to_string();
    app.holder.token = token;
    sync(&mut app.holder);

    // Start the main loop.
    while app.running {
        // Render the user interface.
        tui.draw(&mut app)?;

        // Handle events.
        match tui.events.next()? {
            Event::Tick => app.tick(),
            Event::Key(key_event) => handle_key_events(key_event, &mut app)?,
            Event::Mouse(_) => { },
            Event::Resize(_, _) => { },
        }
    }

    // Exit the user interface.
    tui.exit()?;
    Ok(())
}
