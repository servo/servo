/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use msg::constellation_msg::{BrowsingContextId, PipelineId, TraversalDirection, TopLevelBrowsingContextId};
use script_traits::LoadData;
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
            let SessionHistoryDiff::BrowsingContextDiff(_, ref mut context_diff) = *diff;
            context_diff.replace(old_pipeline_id, new_pipeline_id);
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

    /// The data for the document being loaded.
    pub load_data: LoadData,

    pub replace: Option<PipelineId>,
}

pub struct BrowsingContextChangeset {
    pub pipeline_id: PipelineId,
    pub load_data: LoadData,
}

impl BrowsingContextChangeset {
    pub fn new(diff: &BrowsingContextDiff, direction: TraversalDirection) -> BrowsingContextChangeset {
        match direction {
            TraversalDirection::Forward(_) => {
                BrowsingContextChangeset {
                    pipeline_id: diff.new_pipeline_id,
                    load_data: diff.new_load_data.clone(),
                }
            },
            TraversalDirection::Back(_) => {
                BrowsingContextChangeset {
                    pipeline_id: diff.old_pipeline_id,
                    load_data: diff.old_load_data.clone(),
                }
            }
        }
    }

    pub fn apply_diff(&mut self, diff: &BrowsingContextDiff, direction: TraversalDirection) {
        match direction {
            TraversalDirection::Forward(_) => {
                self.pipeline_id = diff.new_pipeline_id;
                self.load_data = diff.new_load_data.clone();
            },
            TraversalDirection::Back(_) => {
                self.pipeline_id = diff.old_pipeline_id;
                self.load_data = diff.old_load_data.clone();
            }
        }
    }
}

#[derive(Debug)]
pub enum SessionHistoryDiff {
    BrowsingContextDiff(BrowsingContextId, BrowsingContextDiff),
}

#[derive(Debug)]
pub struct BrowsingContextDiff {
    pub browsing_context_id: BrowsingContextId,

    pub old_pipeline_id: PipelineId,

    pub old_load_data: LoadData,

    pub new_pipeline_id: PipelineId,

    pub new_load_data: LoadData,
}

impl BrowsingContextDiff {
    pub fn replace(&mut self, old_pipeline_id: PipelineId, new_pipeline_id: PipelineId) {
        if self.old_pipeline_id == old_pipeline_id {
            self.old_pipeline_id = new_pipeline_id;
        }
        if self.new_pipeline_id == old_pipeline_id {
            self.new_pipeline_id = new_pipeline_id;
        }
    }
}
