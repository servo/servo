/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use euclid::size::TypedSize2D;
use msg::constellation_msg::{BrowsingContextId, TopLevelBrowsingContextId, PipelineId};
use pipeline::Pipeline;
use script_traits::LoadData;
use std::collections::HashMap;
use std::iter::once;
use std::mem::replace;
use std::time::Instant;
use style_traits::CSSPixel;

/// The constellation's view of a browsing context.
/// Each browsing context has a session history, caused by
/// navigation and traversing the history. Each browsing context has its
/// current entry, plus past and future entries. The past is sorted
/// chronologically, the future is sorted reverse chronologically:
/// in particular prev.pop() is the latest past entry, and
/// next.pop() is the earliest future entry.
pub struct BrowsingContext {
    /// The browsing context id.
    pub id: BrowsingContextId,

    /// The top-level browsing context ancestor
    pub top_level_id: TopLevelBrowsingContextId,

    /// The size of the frame.
    pub size: Option<TypedSize2D<f32, CSSPixel>>,

    /// The timestamp for the current session history entry.
    pub instant: Instant,

    /// The pipeline for the current session history entry.
    pub pipeline_id: PipelineId,

    /// The load data for the current session history entry.
    pub load_data: LoadData,

    /// The past session history, ordered chronologically.
    pub prev: Vec<SessionHistoryEntry>,

    /// The future session history, ordered reverse chronologically.
    pub next: Vec<SessionHistoryEntry>,
}

impl BrowsingContext {
    /// Create a new browsing context.
    /// Note this just creates the browsing context, it doesn't add it to the constellation's set of browsing contexts.
    pub fn new(id: BrowsingContextId,
               top_level_id: TopLevelBrowsingContextId,
               pipeline_id: PipelineId,
               load_data: LoadData)
               -> BrowsingContext
    {
        BrowsingContext {
            id: id,
            top_level_id: top_level_id,
            size: None,
            pipeline_id: pipeline_id,
            instant: Instant::now(),
            load_data: load_data,
            prev: vec!(),
            next: vec!(),
        }
    }

    /// Get the current session history entry.
    pub fn current(&self) -> SessionHistoryEntry {
        SessionHistoryEntry {
            instant: self.instant,
            browsing_context_id: self.id,
            pipeline_id: Some(self.pipeline_id),
            load_data: self.load_data.clone(),
        }
    }

    /// Set the current session history entry, and push the current frame entry into the past.
    pub fn load(&mut self, pipeline_id: PipelineId, load_data: LoadData) {
        let current = self.current();
        self.prev.push(current);
        self.instant = Instant::now();
        self.pipeline_id = pipeline_id;
        self.load_data = load_data;
    }

    /// Set the future to be empty.
    pub fn remove_forward_entries(&mut self) -> Vec<SessionHistoryEntry> {
        replace(&mut self.next, vec!())
    }

    /// Update the current entry of the BrowsingContext from an entry that has been traversed to.
    pub fn update_current(&mut self, pipeline_id: PipelineId, entry: SessionHistoryEntry) {
        self.pipeline_id = pipeline_id;
        self.instant = entry.instant;
        self.load_data = entry.load_data;
    }

    /// Is this a top-level browsing context?
    pub fn is_top_level(&self) -> bool {
        self.id == self.top_level_id
    }
}

/// An entry in a browsing context's session history.
/// Each entry stores the pipeline id for a document in the session history.
///
/// When we operate on the joint session history, entries are sorted chronologically,
/// so we timestamp the entries by when the entry was added to the session history.
///
/// https://html.spec.whatwg.org/multipage/#session-history-entry
#[derive(Clone)]
pub struct SessionHistoryEntry {
    /// The timestamp for when the session history entry was created
    pub instant: Instant,

    /// The pipeline for the document in the session history,
    /// None if the entry has been discarded
    pub pipeline_id: Option<PipelineId>,

    /// The load data for this entry, used to reload the pipeline if it has been discarded
    pub load_data: LoadData,

    /// The frame that this session history entry is part of
    pub browsing_context_id: BrowsingContextId,
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

    /// Is the new document replacing the current document (e.g. a reload)
    /// or pushing it into the session history (e.g. a navigation)?
    /// If it is replacing an existing entry, we store its timestamp.
    pub replace_instant: Option<Instant>,
}

/// An iterator over browsing contexts, returning the descendant
/// contexts whose active documents are fully active, in depth-first
/// order.
pub struct FullyActiveBrowsingContextsIterator<'a> {
    /// The browsing contexts still to iterate over.
    pub stack: Vec<BrowsingContextId>,

    /// The set of all browsing contexts.
    pub browsing_contexts: &'a HashMap<BrowsingContextId, BrowsingContext>,

    /// The set of all pipelines.  We use this to find the active
    /// children of a frame, which are the iframes in the currently
    /// active document.
    pub pipelines: &'a HashMap<PipelineId, Pipeline>,
}

impl<'a> Iterator for FullyActiveBrowsingContextsIterator<'a> {
    type Item = &'a BrowsingContext;
    fn next(&mut self) -> Option<&'a BrowsingContext> {
        loop {
            let browsing_context_id = match self.stack.pop() {
                Some(browsing_context_id) => browsing_context_id,
                None => return None,
            };
            let browsing_context = match self.browsing_contexts.get(&browsing_context_id) {
                Some(browsing_context) => browsing_context,
                None => {
                    warn!("BrowsingContext {:?} iterated after closure.", browsing_context_id);
                    continue;
                },
            };
            let pipeline = match self.pipelines.get(&browsing_context.pipeline_id) {
                Some(pipeline) => pipeline,
                None => {
                    warn!("Pipeline {:?} iterated after closure.", browsing_context.pipeline_id);
                    continue;
                },
            };
            self.stack.extend(pipeline.children.iter());
            return Some(browsing_context)
        }
    }
}

/// An iterator over browsing contexts, returning all descendant
/// contexts in depth-first order. Note that this iterator returns all
/// contexts, not just the fully active ones.
pub struct AllBrowsingContextsIterator<'a> {
    /// The browsing contexts still to iterate over.
    pub stack: Vec<BrowsingContextId>,

    /// The set of all browsing contexts.
    pub browsing_contexts: &'a HashMap<BrowsingContextId, BrowsingContext>,

    /// The set of all pipelines.  We use this to find the
    /// children of a browsing context, which are the iframes in all documents
    /// in the session history.
    pub pipelines: &'a HashMap<PipelineId, Pipeline>,
}

impl<'a> Iterator for AllBrowsingContextsIterator<'a> {
    type Item = &'a BrowsingContext;
    fn next(&mut self) -> Option<&'a BrowsingContext> {
        let pipelines = self.pipelines;
        loop {
            let browsing_context_id = match self.stack.pop() {
                Some(browsing_context_id) => browsing_context_id,
                None => return None,
            };
            let browsing_context = match self.browsing_contexts.get(&browsing_context_id) {
                Some(browsing_context) => browsing_context,
                None => {
                    warn!("BrowsingContext {:?} iterated after closure.", browsing_context_id);
                    continue;
                },
            };
            let child_browsing_context_ids = browsing_context.prev.iter().chain(browsing_context.next.iter())
                .filter_map(|entry| entry.pipeline_id)
                .chain(once(browsing_context.pipeline_id))
                .filter_map(|pipeline_id| pipelines.get(&pipeline_id))
                .flat_map(|pipeline| pipeline.children.iter());
            self.stack.extend(child_browsing_context_ids);
            return Some(browsing_context)
        }
    }
}
