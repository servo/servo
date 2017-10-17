/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use cssparser::SourceLocation;
use ipc_channel::ipc::IpcSender;
use log;
use msg::constellation_msg::PipelineId;
use script_traits::ConstellationControlMsg;
use servo_url::ServoUrl;
use std::sync::{Mutex, Arc};
use style::error_reporting::{ParseErrorReporter, ContextualParseError};

#[derive(Clone, MallocSizeOf)]
pub struct CSSErrorReporter {
    pub pipelineid: PipelineId,
    // Arc+Mutex combo is necessary to make this struct Sync,
    // which is necessary to fulfill the bounds required by the
    // uses of the ParseErrorReporter trait.
    #[ignore_malloc_size_of = "Arc is defined in libstd"]
    pub script_chan: Arc<Mutex<IpcSender<ConstellationControlMsg>>>,
}

impl ParseErrorReporter for CSSErrorReporter {
    fn report_error(&self,
                    url: &ServoUrl,
                    location: SourceLocation,
                    error: ContextualParseError) {
        if log_enabled!(log::LogLevel::Info) {
            info!("Url:\t{}\n{}:{} {}",
                  url.as_str(),
                  location.line,
                  location.column,
                  error)
        }

        //TODO: report a real filename
        let _ = self.script_chan.lock().unwrap().send(
            ConstellationControlMsg::ReportCSSError(self.pipelineid,
                                                    "".to_owned(),
                                                    location.line,
                                                    location.column,
                                                    error.to_string()));
    }
}
