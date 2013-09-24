/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */
#[macro_escape];
macro_rules! special_stream(
    ($Chan:ident) => (
        {
            let (port, chan) = stream::();
            (port, $Chan::new(chan))
        }
    );
)
