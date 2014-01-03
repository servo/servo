/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

/// Some little helpers for hooking up the HTML parser with the CSS parser.

use std::comm::Port;
use encoding::EncodingRef;
use encoding::all::UTF_8;
use style::Stylesheet;
use servo_net::resource_task::{Load, LoadResponse, ProgressMsg, Payload, Done, ResourceTask};
use servo_util::task::spawn_named;
use extra::url::Url;

/// Where a style sheet comes from.
pub enum StylesheetProvenance {
    UrlProvenance(Url),
    InlineProvenance(Url, ~str),
}

pub fn spawn_css_parser(provenance: StylesheetProvenance,
                        resource_task: ResourceTask)
                     -> Port<Stylesheet> {
    let (result_port, result_chan) = Chan::new();

    // TODO: Get the actual value. http://dev.w3.org/csswg/css-syntax/#environment-encoding
    let environment_encoding = UTF_8 as EncodingRef;

    spawn_named("cssparser", proc() {
        // TODO: CSS parsing should take a base URL.
        let _url = match provenance {
            UrlProvenance(ref the_url) => (*the_url).clone(),
            InlineProvenance(ref the_url, _) => (*the_url).clone()
        };

        let sheet = match provenance {
            UrlProvenance(url) => {
                debug!("cssparse: loading style sheet at {:s}", url.to_str());
                let (input_port, input_chan) = Chan::new();
                resource_task.send(Load(url, input_chan));
                let LoadResponse { metadata: metadata, progress_port: progress_port }
                    = input_port.recv();
                let protocol_encoding_label = metadata.charset.as_ref().map(|s| s.as_slice());
                let iter = ProgressMsgPortIterator { progress_port: progress_port };
                Stylesheet::from_bytes_iter(
                    iter, metadata.final_url,
                    protocol_encoding_label, Some(environment_encoding))
            }
            InlineProvenance(base_url, data) => {
                Stylesheet::from_str(data, base_url, environment_encoding)
            }
        };
        result_chan.send(sheet);
    });

    return result_port;
}

struct ProgressMsgPortIterator {
    progress_port: Port<ProgressMsg>
}

impl Iterator<~[u8]> for ProgressMsgPortIterator {
    fn next(&mut self) -> Option<~[u8]> {
        match self.progress_port.recv() {
            Payload(data) => Some(data),
            Done(..) => None
        }
    }
}
