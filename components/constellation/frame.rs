/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use msg::constellation_msg::{FrameId, PipelineId};
use pipeline::Pipeline;
use std::collections::HashMap;
use std::iter::once;
use std::mem::replace;
use std::time::Instant;

/// A frame in the frame tree.
/// Each frame is the constrellation's view of a browsing context.
/// Each browsing context has a session history, caused by
/// navigation and traversing the history. Each frame has its
/// current entry, plus past and future entries. The past is sorted
/// chronologically, the future is sorted reverse chronoogically:
/// in partiucular prev.pop() is the latest past entry, and
/// next.pop() is the earliest future entry.
#[derive(Debug, Clone)]
pub struct Frame {
    /// The frame id.
    pub id: FrameId,

    /// The past session history, ordered chronologically.
    pub prev: Vec<FrameState>,

    /// The currently active session history entry.
    pub current: FrameState,

    /// The future session history, ordered reverse chronologically.
    pub next: Vec<FrameState>,
}

impl Frame {
    /// Create a new frame.
    /// Note this just creates the frame, it doesn't add it to the frame tree.
    pub fn new(id: FrameId, pipeline_id: PipelineId) -> Frame {
        Frame {
            id: id,
            prev: vec!(),
            current: FrameState::new(pipeline_id, id),
            next: vec!(),
        }
    }

    /// Set the current frame entry, and push the current frame entry into the past.
    pub fn load(&mut self, pipeline_id: PipelineId) {
        self.prev.push(self.current.clone());
        self.current = FrameState::new(pipeline_id, self.id);
    }

    /// Set the future to be empty.
    pub fn remove_forward_entries(&mut self) -> Vec<FrameState> {
        replace(&mut self.next, vec!())
    }

    /// Set the current frame entry, and drop the current frame entry.
    pub fn replace_current(&mut self, pipeline_id: PipelineId) -> FrameState {
        replace(&mut self.current, FrameState::new(pipeline_id, self.id))
    }
}

/// An entry in a frame's session history.
/// Each entry stores the pipeline id for a document in the session history.
///
/// When we operate on the joint session history, entries are sorted chronologically,
/// so we timestamp the entries by when the entry was added to the session history.
#[derive(Debug, Clone)]
pub struct FrameState {
    /// The timestamp for when the session history entry was created
    pub instant: Instant,
    /// The pipeline for the document in the session history
    pub pipeline_id: PipelineId,
    /// The frame that this session history entry is part of
    pub frame_id: FrameId,
}

impl FrameState {
    /// Create a new session history entry.
    fn new(pipeline_id: PipelineId, frame_id: FrameId) -> FrameState {
        FrameState {
            instant: Instant::now(),
            pipeline_id: pipeline_id,
            frame_id: frame_id,
        }
    }
}

/// Represents a pending change in the frame tree, that will be applied
/// once the new pipeline has loaded and completed initial layout / paint.
pub struct FrameChange {
    /// The frame to change.
    pub frame_id: FrameId,

    /// The pipeline that was currently active at the time the change started.
    /// TODO: can this field be removed?
    pub old_pipeline_id: Option<PipelineId>,

    /// The pipeline for the document being loaded.
    pub new_pipeline_id: PipelineId,

    /// Is the new document replacing the current document (e.g. a reload)
    /// or pushing it into the session history (e.g. a navigation)?
    pub replace: bool,
}

/// An iterator over a frame tree, returning the fully active frames in
/// depth-first order. Note that this iterator only returns the fully
/// active frames, that is ones where every ancestor frame is
/// in the currently active pipeline of its parent frame.
pub struct FrameTreeIterator<'a> {
    /// The frames still to iterate over.
    pub stack: Vec<FrameId>,

    /// The set of all frames.
    pub frames: &'a HashMap<FrameId, Frame>,

    /// The set of all pipelines.  We use this to find the active
    /// children of a frame, which are the iframes in the currently
    /// active document.
    pub pipelines: &'a HashMap<PipelineId, Pipeline>,
}

impl<'a> Iterator for FrameTreeIterator<'a> {
    type Item = &'a Frame;
    fn next(&mut self) -> Option<&'a Frame> {
        loop {
            let frame_id = match self.stack.pop() {
                Some(frame_id) => frame_id,
                None => return None,
            };
            let frame = match self.frames.get(&frame_id) {
                Some(frame) => frame,
                None => {
                    warn!("Frame {:?} iterated after closure.", frame_id);
                    continue;
                },
            };
            let pipeline = match self.pipelines.get(&frame.current.pipeline_id) {
                Some(pipeline) => pipeline,
                None => {
                    warn!("Pipeline {:?} iterated after closure.", frame.current.pipeline_id);
                    continue;
                },
            };
            self.stack.extend(pipeline.children.iter().map(|&c| c));
            return Some(frame)
        }
    }
}

/// An iterator over a frame tree, returning all frames in depth-first
/// order. Note that this iterator returns all frames, not just the
/// fully active ones.
pub struct FullFrameTreeIterator<'a> {
    /// The frames still to iterate over.
    pub stack: Vec<FrameId>,

    /// The set of all frames.
    pub frames: &'a HashMap<FrameId, Frame>,

    /// The set of all pipelines.  We use this to find the
    /// children of a frame, which are the iframes in all documents
    /// in the session history.
    pub pipelines: &'a HashMap<PipelineId, Pipeline>,
}

impl<'a> Iterator for FullFrameTreeIterator<'a> {
    type Item = &'a Frame;
    fn next(&mut self) -> Option<&'a Frame> {
        loop {
            let frame_id = match self.stack.pop() {
                Some(frame_id) => frame_id,
                None => return None,
            };
            let frame = match self.frames.get(&frame_id) {
                Some(frame) => frame,
                None => {
                    warn!("Frame {:?} iterated after closure.", frame_id);
                    continue;
                },
            };
            for entry in frame.prev.iter().chain(frame.next.iter()).chain(once(&frame.current)) {
                if let Some(pipeline) = self.pipelines.get(&entry.pipeline_id) {
                    self.stack.extend(pipeline.children.iter().map(|&c| c));
                }
            }
            return Some(frame)
        }
    }
}
