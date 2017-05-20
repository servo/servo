/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use cssparser::{Parser, SourcePosition};
use ipc_channel::ipc::IpcSender;
use log;
use msg::constellation_msg::PipelineId;
use script_traits::ConstellationControlMsg;
use servo_url::ServoUrl;
use std::sync::{Mutex, Arc};
use style::error_reporting::ParseErrorReporter;

#[derive(HeapSizeOf, Clone)]
pub struct CSSErrorReporter {
    pub pipelineid: PipelineId,
    // Arc+Mutex combo is necessary to make this struct Sync,
    // which is necessary to fulfill the bounds required by the
    // uses of the ParseErrorReporter trait.
    #[ignore_heap_size_of = "Arc is defined in libstd"]
    pub script_chan: Arc<Mutex<IpcSender<ConstellationControlMsg>>>,
}

impl ParseErrorReporter for CSSErrorReporter {
    fn report_error(&self,
                    input: &mut Parser,
                    position: SourcePosition,
                    message: &str,
                    url: &ServoUrl,
                    line_number_offset: u64) {
        let location = input.source_location(position);
        let line_offset = location.line + line_number_offset as usize;
        if log_enabled!(log::LogLevel::Info) {
            info!("Url:\t{}\n{}:{} {}",
                  url.as_str(),
                  line_offset,
                  location.column,
                  message)
        }

        //TODO: report a real filename
        let _ = self.script_chan.lock().unwrap().send(
            ConstellationControlMsg::ReportCSSError(self.pipelineid,
                                                    "".to_owned(),
                                                    location.line,
                                                    location.column,
                                                    message.to_owned()));
    }
}
