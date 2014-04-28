/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

/// Some little helpers for hooking up the HTML parser with the CSS parser.

use std::comm::{channel, Receiver};
use encoding::EncodingRef;
use encoding::all::UTF_8;
use style::Stylesheet;
use servo_net::resource_task::{Load, LoadResponse, ProgressMsg, Payload, Done, ResourceTask};
use servo_util::task::spawn_named;
use url::Url;

/// Where a style sheet comes from.
pub enum StylesheetProvenance {
    UrlProvenance(Url, ResourceTask),
    InlineProvenance(Url, ~str),
}

// Parses the style data and returns the stylesheet
pub fn parse_inline_css(url: Url, data: ~str) -> Stylesheet {
    parse_css(InlineProvenance(url, data))
}

fn parse_css(provenance: StylesheetProvenance) -> Stylesheet {
    // TODO: Get the actual value. http://dev.w3.org/csswg/css-syntax/#environment-encoding
    let environment_encoding = UTF_8 as EncodingRef;

    match provenance {
        UrlProvenance(url, resource_task) => {
            debug!("cssparse: loading style sheet at {:s}", url.to_str());
            let (input_chan, input_port) = channel();
            resource_task.send(Load(url, input_chan));
            let LoadResponse { metadata: metadata, progress_port: progress_port }
                = input_port.recv();
            let final_url = &metadata.final_url;
            let protocol_encoding_label = metadata.charset.as_ref().map(|s| s.as_slice());
            let iter = ProgressMsgPortIterator { progress_port: progress_port };
            Stylesheet::from_bytes_iter(
                iter, final_url.clone(),
                protocol_encoding_label, Some(environment_encoding))
        }
        InlineProvenance(base_url, data) => {
            debug!("cssparse: loading inline stylesheet {:s}", data);
            Stylesheet::from_str(data, base_url, environment_encoding)
        }
    }
}

pub fn spawn_css_parser(provenance: StylesheetProvenance) -> Receiver<Stylesheet> {
    let (result_chan, result_port) = channel();

    spawn_named("cssparser", proc() {
        result_chan.send(parse_css(provenance));
    });

    return result_port;
}

struct ProgressMsgPortIterator {
    progress_port: Receiver<ProgressMsg>
}

impl Iterator<Vec<u8>> for ProgressMsgPortIterator {
    fn next(&mut self) -> Option<Vec<u8>> {
        match self.progress_port.recv() {
            Payload(data) => Some(data),
            Done(..) => None
        }
    }
}
