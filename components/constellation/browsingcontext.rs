/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use std::collections::{HashMap, HashSet};

use base::id::{BrowsingContextGroupId, BrowsingContextId, PipelineId, TopLevelBrowsingContextId};
use euclid::Size2D;
use log::warn;
use style_traits::CSSPixel;

use crate::pipeline::Pipeline;

/// Because a browsing context is only constructed once the document that's
/// going to be in it becomes active (i.e. not when a pipeline is spawned), some
/// values needed in browsing context are not easily available at the point of
/// constructing it. Thus, every time a pipeline is created for a browsing
/// context which doesn't exist yet, these values needed for the new browsing
/// context are stored here so that they may be available later.
pub struct NewBrowsingContextInfo {
    /// The parent pipeline that contains this browsing context. `None` if this
    /// is a top level browsing context.
    pub parent_pipeline_id: Option<PipelineId>,

    /// Whether this browsing context is in private browsing mode.
    pub is_private: bool,

    /// Whether this browsing context inherits a secure context.
    pub inherited_secure_context: Option<bool>,

    /// Whether this browsing context should be throttled, using less resources
    /// by stopping animations and running timers at a heavily limited rate.
    pub throttled: bool,
}

/// The constellation's view of a browsing context.
/// Each browsing context has a session history, caused by navigation and
/// traversing the history. Each browsing context has its current entry, plus
/// past and future entries. The past is sorted chronologically, the future is
/// sorted reverse chronologically: in particular prev.pop() is the latest
/// past entry, and next.pop() is the earliest future entry.
pub struct BrowsingContext {
    /// The browsing context group id where the top-level of this bc is found.
    pub bc_group_id: BrowsingContextGroupId,

    /// The browsing context id.
    pub id: BrowsingContextId,

    /// The top-level browsing context ancestor
    pub top_level_id: TopLevelBrowsingContextId,

    /// The size of the frame.
    pub size: Size2D<f32, CSSPixel>,

    /// Whether this browsing context is in private browsing mode.
    pub is_private: bool,

    /// Whether this browsing context inherits a secure context.
    pub inherited_secure_context: Option<bool>,

    /// Whether this browsing context should be throttled, using less resources
    /// by stopping animations and running timers at a heavily limited rate.
    pub throttled: bool,

    /// The pipeline for the current session history entry.
    pub pipeline_id: PipelineId,

    /// The parent pipeline that contains this browsing context. `None` if this
    /// is a top level browsing context.
    pub parent_pipeline_id: Option<PipelineId>,

    /// All the pipelines that have been presented or will be presented in
    /// this browsing context.
    pub pipelines: HashSet<PipelineId>,
}

impl BrowsingContext {
    /// Create a new browsing context.
    /// Note this just creates the browsing context, it doesn't add it to the constellation's set of browsing contexts.
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        bc_group_id: BrowsingContextGroupId,
        id: BrowsingContextId,
        top_level_id: TopLevelBrowsingContextId,
        pipeline_id: PipelineId,
        parent_pipeline_id: Option<PipelineId>,
        size: Size2D<f32, CSSPixel>,
        is_private: bool,
        inherited_secure_context: Option<bool>,
        throttled: bool,
    ) -> BrowsingContext {
        let mut pipelines = HashSet::new();
        pipelines.insert(pipeline_id);
        BrowsingContext {
            bc_group_id,
            id,
            top_level_id,
            size,
            is_private,
            inherited_secure_context,
            throttled,
            pipeline_id,
            parent_pipeline_id,
            pipelines,
        }
    }

    pub fn update_current_entry(&mut self, pipeline_id: PipelineId) {
        self.pipeline_id = pipeline_id;
    }

    /// Is this a top-level browsing context?
    pub fn is_top_level(&self) -> bool {
        self.id == self.top_level_id
    }
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
                    warn!(
                        "BrowsingContext {:?} iterated after closure.",
                        browsing_context_id
                    );
                    continue;
                },
            };
            let pipeline = match self.pipelines.get(&browsing_context.pipeline_id) {
                Some(pipeline) => pipeline,
                None => {
                    warn!(
                        "Pipeline {:?} iterated after closure.",
                        browsing_context.pipeline_id
                    );
                    continue;
                },
            };
            self.stack.extend(pipeline.children.iter());
            return Some(browsing_context);
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
            let browsing_context_id = self.stack.pop()?;
            let browsing_context = match self.browsing_contexts.get(&browsing_context_id) {
                Some(browsing_context) => browsing_context,
                None => {
                    warn!(
                        "BrowsingContext {:?} iterated after closure.",
                        browsing_context_id
                    );
                    continue;
                },
            };
            let child_browsing_context_ids = browsing_context
                .pipelines
                .iter()
                .filter_map(|pipeline_id| pipelines.get(pipeline_id))
                .flat_map(|pipeline| pipeline.children.iter());
            self.stack.extend(child_browsing_context_ids);
            return Some(browsing_context);
        }
    }
}
