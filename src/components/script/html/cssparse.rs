/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

/// Some little helpers for hooking up the HTML parser with the CSS parser.

use core::cell::Cell;
use core::comm::Port;
use core::str;
use newcss::stylesheet::Stylesheet;
use newcss::util::DataStream;
use servo_net::resource_task::{ResourceTask, ProgressMsg, Load, Payload, Done};
use std::net::url::Url;

/// Where a style sheet comes from.
pub enum StylesheetProvenance {
    UrlProvenance(Url),
    InlineProvenance(Url, ~str),
}

pub fn spawn_css_parser(provenance: StylesheetProvenance,
                        resource_task: ResourceTask)
                     -> Port<Stylesheet> {
    let (result_port, result_chan) = comm::stream();

    let provenance_cell = Cell(provenance);
    do task::spawn {
        let url = do provenance_cell.with_ref |p| {
            match *p {
                UrlProvenance(copy the_url) => the_url,
                InlineProvenance(copy the_url, _) => the_url
            }
        };

        let sheet = Stylesheet::new(url, data_stream(provenance_cell.take(),
                                                     resource_task.clone()));
        result_chan.send(sheet);
    }

    return result_port;
}

fn data_stream(provenance: StylesheetProvenance, resource_task: ResourceTask) -> DataStream {
    match provenance {
        UrlProvenance(url) => {
            debug!("cssparse: loading style sheet at %s", url.to_str());
            let (input_port, input_chan) = comm::stream();
            resource_task.send(Load(url, input_chan));
            resource_port_to_data_stream(input_port)
        }
        InlineProvenance(_, data) => {
            data_to_data_stream(data)
        }
    }
}

fn resource_port_to_data_stream(input_port: Port<ProgressMsg>) -> DataStream {
    return || {
        match input_port.recv() {
            Payload(data) => Some(data),
            Done(*) => None
        }
    }
}

fn data_to_data_stream(data: ~str) -> DataStream {
    let data_cell = Cell(data);
    return || {
        if data_cell.is_empty() {
            None
        } else {
            // FIXME: Blech, a copy.
            Some(str::to_bytes(data_cell.take()))
        }
    }
}

