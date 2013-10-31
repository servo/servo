/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

/// Some little helpers for hooking up the HTML parser with the CSS parser.

use std::cell::Cell;
use std::comm;
use std::comm::Port;
use std::task;
use style::Stylesheet;
use servo_net::resource_task::{Load, ProgressMsg, Payload, Done, ResourceTask};
use extra::url::Url;

/// Where a style sheet comes from.
pub enum StylesheetProvenance {
    UrlProvenance(Url),
    InlineProvenance(Url, ~str),
}

pub fn spawn_css_parser(provenance: StylesheetProvenance,
                        resource_task: ResourceTask)
                     -> Port<Stylesheet> {
    let (result_port, result_chan) = comm::stream();

    let provenance_cell = Cell::new(provenance);
    do task::spawn {
        // TODO: CSS parsing should take a base URL.
        let _url = do provenance_cell.with_ref |p| {
            match *p {
                UrlProvenance(ref the_url) => (*the_url).clone(),
                InlineProvenance(ref the_url, _) => (*the_url).clone()
            }
        };

        let sheet = match provenance_cell.take() {
            UrlProvenance(url) => {
                debug!("cssparse: loading style sheet at {:s}", url.to_str());
                let (input_port, input_chan) = comm::stream();
                resource_task.send(Load(url, input_chan));
                Stylesheet::from_iter(ProgressMsgPortIterator {
                    progress_port: input_port.recv().progress_port
                })
            }
            InlineProvenance(_, data) => {
                Stylesheet::from_str(data)
            }
        };
        result_chan.send(sheet);
    }

    return result_port;
}

struct ProgressMsgPortIterator {
    progress_port: Port<ProgressMsg>
}

impl Iterator<~[u8]> for ProgressMsgPortIterator {
    fn next(&mut self) -> Option<~[u8]> {
        match self.progress_port.recv() {
            Payload(data) => Some(data),
            Done(*) => None
        }
    }
}
