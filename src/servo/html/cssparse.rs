/*!
Some little helpers for hooking up the HTML parser with the CSS parser
*/

use resource::resource_task::{ResourceTask, ProgressMsg, Load, Payload, Done};

use core::pipes::{Port, Chan};
use core::pipes;
use core::str;
use newcss::stylesheet::Stylesheet;
use newcss::util::DataStream;
use std::cell::Cell;
use std::net::url::Url;
use std::net::url;

/// Where a style sheet comes from.
pub enum StylesheetProvenance {
    UrlProvenance(Url),
    InlineProvenance(Url, ~str),
}

pub fn spawn_css_parser(provenance: StylesheetProvenance,
                        resource_task: ResourceTask)
                     -> Port<Stylesheet> {
    let (result_port, result_chan) = pipes::stream();

    let provenance_cell = Cell(move provenance);
    do task::spawn |move provenance_cell, copy resource_task| {
        let url = do provenance_cell.with_ref |p| {
            match *p {
                UrlProvenance(copy the_url) => move the_url,
                InlineProvenance(copy the_url, _) => move the_url
            }
        };

        let sheet = Stylesheet::new(url, data_stream(provenance_cell.take(),
                                                     resource_task.clone()));
        result_chan.send(sheet);
    }

    return result_port;
}

fn data_stream(provenance: StylesheetProvenance, resource_task: ResourceTask) -> DataStream {
    match move provenance {
        UrlProvenance(move url) => {
            let (input_port, input_chan) = pipes::stream();
            resource_task.send(Load(move url, input_chan));
            resource_port_to_data_stream(input_port)
        }
        InlineProvenance(_, move data) => {
            data_to_data_stream(move data)
        }
    }
}

fn resource_port_to_data_stream(input_port: Port<ProgressMsg>) -> DataStream {
    return || {
        match input_port.recv() {
            Payload(move data) => Some(move data),
            Done(*) => None
        }
    }
}

fn data_to_data_stream(data: ~str) -> DataStream {
    let data_cell = Cell(move data);
    return |move data_cell| {
        if data_cell.is_empty() {
            None
        } else {
            // FIXME: Blech, a copy.
            Some(str::to_bytes(data_cell.take()))
        }
    }
}

