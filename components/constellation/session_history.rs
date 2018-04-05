/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use msg::constellation_msg::{BrowsingContextId, PipelineId, TopLevelBrowsingContextId};
use script_traits::LoadData;
use std::{fmt, mem};
use std::cmp::PartialEq;

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

    pub fn replace(&mut self, old_pipeline_id: AliveOrDeadPipeline, new_pipeline_id: AliveOrDeadPipeline) {
        for diff in self.past.iter_mut().chain(self.future.iter_mut()) {
            diff.replace(&old_pipeline_id, &new_pipeline_id);
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

    pub replace: Option<AliveOrDeadPipeline>,
}

/// Represents a pipeline or discarded pipeline in a history entry.
#[derive(Clone, Debug)]
pub enum AliveOrDeadPipeline {
    /// Represents a pipeline that has not been discarded
    Alive(PipelineId),
    /// Represents a pipeline that has been discarded and must be reloaded with the given `LoadData`
    Dead(PipelineId, LoadData),
}

impl fmt::Display for AliveOrDeadPipeline {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            AliveOrDeadPipeline::Alive(pipeline_id) => write!(fmt, "Alive({})", pipeline_id),
            AliveOrDeadPipeline::Dead(pipeline_id, ..) => write!(fmt, "Dead({})", pipeline_id),
        }
    }
}

impl AliveOrDeadPipeline {
    pub fn alive_pipeline_id(&self) -> Option<PipelineId> {
        match *self {
            AliveOrDeadPipeline::Alive(pipeline_id) => Some(pipeline_id),
            AliveOrDeadPipeline::Dead(..) => None,
        }
    }
}

impl PartialEq for AliveOrDeadPipeline {
    fn eq(&self, other: &AliveOrDeadPipeline) -> bool {
        match *self {
            AliveOrDeadPipeline::Alive(pipeline_id) => {
                match *other {
                    AliveOrDeadPipeline::Alive(other_pipeline_id) if pipeline_id == other_pipeline_id => true,
                    _ => false,
                }
            },
            AliveOrDeadPipeline::Dead(pipeline_id, _) => {
                match *other {
                    AliveOrDeadPipeline::Dead(other_pipeline_id, _) if pipeline_id == other_pipeline_id => true,
                    _ => false,
                }
            }
        }
    }
}

#[derive(Debug)]
pub enum SessionHistoryDiff {
    BrowsingContextDiff {
        browsing_context_id: BrowsingContextId,
        old_pipeline_id: AliveOrDeadPipeline,
        new_pipeline_id: AliveOrDeadPipeline,
    },
}

impl SessionHistoryDiff {
    pub fn alive_old_pipeline(&self) -> Option<PipelineId> {
        match *self {
            SessionHistoryDiff::BrowsingContextDiff { ref old_pipeline_id, .. } => {
                match *old_pipeline_id {
                    AliveOrDeadPipeline::Alive(pipeline_id) => Some(pipeline_id),
                    AliveOrDeadPipeline::Dead(..) => None,
                }
            }
        }
    }

    pub fn alive_new_pipeline(&self) -> Option<PipelineId> {
        match *self {
            SessionHistoryDiff::BrowsingContextDiff { ref new_pipeline_id, .. } => {
                match *new_pipeline_id {
                    AliveOrDeadPipeline::Alive(pipeline_id) => Some(pipeline_id),
                    AliveOrDeadPipeline::Dead(..) => None,
                }
            }
        }
    }

    pub fn replace(&mut self, stale_pipeline_id: &AliveOrDeadPipeline, pipeline_id: &AliveOrDeadPipeline) {
        match *self {
            SessionHistoryDiff::BrowsingContextDiff { ref mut old_pipeline_id, ref mut new_pipeline_id, .. } => {
                if *old_pipeline_id == *stale_pipeline_id {
                    *old_pipeline_id = pipeline_id.clone();
                }
                if *new_pipeline_id == *stale_pipeline_id {
                    *new_pipeline_id = pipeline_id.clone();
                }
            }
        }
    }
}
