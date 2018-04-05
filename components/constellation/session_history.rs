/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use msg::constellation_msg::{BrowsingContextId, PipelineId, TopLevelBrowsingContextId};
use std::mem;

#[derive(Debug)]
pub struct SessionHistory {
    pub past: Vec<SessionHistoryDiff>,

    pub future: Vec<SessionHistoryDiff>,
}

impl SessionHistory {
    pub fn new() -> SessionHistory {
        SessionHistory {
            past: Vec::new(),
            future: Vec::new(),
        }
    }

    pub fn history_length(&self) -> usize {
        self.past.len() + 1 + self.future.len()
    }

    pub fn push_diff(&mut self, diff: SessionHistoryDiff) -> Vec<SessionHistoryDiff> {
        self.past.push(diff);
        mem::replace(&mut self.future, vec![])
    }

    pub fn replace(&mut self, old_pipeline_id: PipelineId, new_pipeline_id: PipelineId) {
        for diff in self.past.iter_mut().chain(self.future.iter_mut()) {
            diff.replace(old_pipeline_id, new_pipeline_id);
        }
    }
}

/// Represents a pending change in a session history, that will be applied
/// once the new pipeline has loaded and completed initial layout / paint.
pub struct SessionHistoryChange {
    /// The browsing context to change.
    pub browsing_context_id: BrowsingContextId,

    /// The top-level browsing context ancestor.
    pub top_level_browsing_context_id: TopLevelBrowsingContextId,

    /// The pipeline for the document being loaded.
    pub new_pipeline_id: PipelineId,

    pub replace: Option<PipelineId>,
}

#[derive(Debug)]
pub enum SessionHistoryDiff {
    BrowsingContextDiff {
        browsing_context_id: BrowsingContextId,
        old_pipeline_id: PipelineId,
        new_pipeline_id: PipelineId,
    },
}

impl SessionHistoryDiff {
    pub fn replace(&mut self, stale_pipeline_id: PipelineId, pipeline_id: PipelineId) {
        match *self {
            SessionHistoryDiff::BrowsingContextDiff { ref mut old_pipeline_id, ref mut new_pipeline_id, .. } => {
                if *old_pipeline_id == stale_pipeline_id {
                    *old_pipeline_id = pipeline_id;
                }
                if *new_pipeline_id == stale_pipeline_id {
                    *new_pipeline_id = pipeline_id;
                }
            }
        }
    }
}
