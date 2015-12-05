/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use cssparser::{Parser, SourcePosition};
use log;
use msg::ParseErrorReporter;
use msg::constellation_msg::PipelineId;

#[derive(JSTraceable, HeapSizeOf)]
pub struct CSSErrorReporter {
    pub pipelineid: PipelineId,
}

impl ParseErrorReporter for CSSErrorReporter {
     fn report_error(&self, input: &mut Parser, position: SourcePosition, message: &str) {
         if log_enabled!(log::LogLevel::Info) {
             let location = input.source_location(position);
             // TODO eventually this will got into a "web console" or something.
             info!("{}:{} {}", location.line, location.column, message)
         }
     }

     fn clone(&self) -> Box<ParseErrorReporter + Send + Sync> {
         let error_reporter = box CSSErrorReporter { pipelineid: self.pipelineid, } ;
         return error_reporter;
     }
     fn pipeline(&self) -> PipelineId {
         return self.pipelineid;
     }
}
