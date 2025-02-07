/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use std::{env, panic};

use crate::desktop::app::App;
use crate::desktop::events_loop::EventsLoop;
use crate::panic_hook;
use crate::prefs::{parse_command_line_arguments, ArgumentParsingResult};

pub fn main() {
    crate::crash_handler::install();
    crate::init_crypto();
    crate::resources::init();

    // TODO: once log-panics is released, can this be replaced by
    // log_panics::init()?
    panic::set_hook(Box::new(panic_hook::panic_hook));

    let args = env::args().collect();
    let (opts, preferences, servoshell_preferences) = match parse_command_line_arguments(args) {
        ArgumentParsingResult::ContentProcess(token) => return servo::run_content_process(token),
        ArgumentParsingResult::ChromeProcess(opts, preferences, servoshell_preferences) => {
            (opts, preferences, servoshell_preferences)
        },
    };

    crate::init_tracing(servoshell_preferences.tracing_filter.as_deref());

    let clean_shutdown = servoshell_preferences.clean_shutdown;
    let event_loop = EventsLoop::new(servoshell_preferences.headless, opts.output_file.is_some())
        .expect("Failed to create events loop");

    {
        let mut app = App::new(opts, preferences, servoshell_preferences, &event_loop);
        event_loop.run_app(&mut app);
    }

    crate::platform::deinit(clean_shutdown)
}
