/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use euclid::TypedSize2D;
use msg::constellation_msg::{BrowsingContextId, TopLevelBrowsingContextId, PipelineId};
use pipeline::Pipeline;
use script_traits::LoadData;
use std::collections::HashMap;
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

    /// The pipeline for the current session history entry.
    pub pipeline_id: PipelineId,

    /// The load data for the current session history entry.
    pub load_data: LoadData,
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
            load_data: load_data
        }
    }

    pub fn update_current_entry(&mut self, pipeline_id: PipelineId, load_data: LoadData) {
        self.pipeline_id = pipeline_id;
        self.load_data = load_data;
    }

    /// Is this a top-level browsing context?
    pub fn is_top_level(&self) -> bool {
        self.id == self.top_level_id
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
            let browsing_context_id = self.stack.pop()?;
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
        // TODO: Get all browsing
        // let pipelines = self.pipelines;
        // loop {
        //     let browsing_context_id = self.stack.pop()?;
        //     let browsing_context = match self.browsing_contexts.get(&browsing_context_id) {
        //         Some(browsing_context) => browsing_context,
        //         None => {
        //             warn!("BrowsingContext {:?} iterated after closure.", browsing_context_id);
        //             continue;
        //         },
        //     };
        //     let child_browsing_context_ids = browsing_context.prev.iter().chain(browsing_context.next.iter())
        //         .filter_map(|entry| entry.pipeline_id)
        //         .chain(once(browsing_context.pipeline_id))
        //         .filter_map(|pipeline_id| pipelines.get(&pipeline_id))
        //         .flat_map(|pipeline| pipeline.children.iter());
        //     self.stack.extend(child_browsing_context_ids);
        //     return Some(browsing_context)
        // }
        None
    }
}
