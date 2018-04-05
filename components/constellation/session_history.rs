/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use msg::constellation_msg::{BrowsingContextId, PipelineId, TopLevelBrowsingContextId};
use script_traits::LoadData;
use std::{fmt, mem};
use std::cmp::PartialEq;

/// Represents the joint session history
/// https://html.spec.whatwg.org/multipage/#joint-session-history
#[derive(Debug)]
pub struct JointSessionHistory {
    /// Diffs used to traverse to past entries. Oldest entries are at the back,
    /// the most recent entries are at the front.
    pub past: Vec<SessionHistoryDiff>,

    /// Diffs used to traverse to future entries. Oldest entries are at the back,
    /// the most recent entries are at the front.
    pub future: Vec<SessionHistoryDiff>,
}

impl JointSessionHistory {
    pub fn new() -> JointSessionHistory {
        JointSessionHistory {
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

    pub fn replace(&mut self, old_reloader: NeedsToReload, new_reloader: NeedsToReload) {
        for diff in self.past.iter_mut().chain(self.future.iter_mut()) {
            diff.replace(&old_reloader, &new_reloader);
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

    /// The old pipeline that the new pipeline should replace.
    pub replace: Option<NeedsToReload>,
}

/// Represents a pipeline or discarded pipeline in a history entry.
#[derive(Clone, Debug)]
pub enum NeedsToReload {
    /// Represents a pipeline that has not been discarded
    No(PipelineId),
    /// Represents a pipeline that has been discarded and must be reloaded with the given `LoadData`
    /// if ever traversed to.
    Yes(PipelineId, LoadData),
}

impl fmt::Display for NeedsToReload {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            NeedsToReload::No(pipeline_id) => write!(fmt, "Alive({})", pipeline_id),
            NeedsToReload::Yes(pipeline_id, ..) => write!(fmt, "Dead({})", pipeline_id),
        }
    }
}

impl NeedsToReload {
    pub fn alive_pipeline_id(&self) -> Option<PipelineId> {
        match *self {
            NeedsToReload::No(pipeline_id) => Some(pipeline_id),
            NeedsToReload::Yes(..) => None,
        }
    }
}

// Custom `PartialEq` that only compares the `PipelineId`s of the same variants while ignoring `LoadData`
impl PartialEq for NeedsToReload {
    fn eq(&self, other: &NeedsToReload) -> bool {
        match *self {
            NeedsToReload::No(pipeline_id) => {
                match *other {
                    NeedsToReload::No(other_pipeline_id) => pipeline_id == other_pipeline_id,
                    _ => false,
                }
            },
            NeedsToReload::Yes(pipeline_id, _) => {
                match *other {
                    NeedsToReload::Yes(other_pipeline_id, _) => pipeline_id == other_pipeline_id,
                    _ => false,
                }
            }
        }
    }
}


/// Represents a the difference between two adjacent session history entries.
#[derive(Debug)]
pub enum SessionHistoryDiff {
    /// Represents a diff where the active pipeline of an entry changed.
    BrowsingContextDiff {
        /// The browsing context whose pipeline changed
        browsing_context_id: BrowsingContextId,
        /// The previous pipeline (used when traversing into the past)
        old_reloader: NeedsToReload,
        /// The next pipeline (used when traversing into the future)
        new_reloader: NeedsToReload,
    },
}

impl SessionHistoryDiff {
    /// Returns the old pipeline id if that pipeline is still alive, otherwise returns `None`
    pub fn alive_old_pipeline(&self) -> Option<PipelineId> {
        match *self {
            SessionHistoryDiff::BrowsingContextDiff { ref old_reloader, .. } => {
                match *old_reloader {
                    NeedsToReload::No(pipeline_id) => Some(pipeline_id),
                    NeedsToReload::Yes(..) => None,
                }
            }
        }
    }

    /// Returns the new pipeline id if that pipeline is still alive, otherwise returns `None`
    pub fn alive_new_pipeline(&self) -> Option<PipelineId> {
        match *self {
            SessionHistoryDiff::BrowsingContextDiff { ref new_reloader, .. } => {
                match *new_reloader {
                    NeedsToReload::No(pipeline_id) => Some(pipeline_id),
                    NeedsToReload::Yes(..) => None,
                }
            }
        }
    }

    /// Replaces all occurances of the replaced pipeline with a new pipeline
    pub fn replace(&mut self, replaced_reloader: &NeedsToReload, reloader: &NeedsToReload) {
        match *self {
            SessionHistoryDiff::BrowsingContextDiff { ref mut old_reloader, ref mut new_reloader, .. } => {
                if *old_reloader == *replaced_reloader {
                    *old_reloader = reloader.clone();
                }
                if *new_reloader == *replaced_reloader {
                    *new_reloader = reloader.clone();
                }
            }
        }
    }
}
