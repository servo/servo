/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use constellation::Frame;
use msg::constellation_msg::{PipelineId, PipelineNamespace, PipelineNamespaceId, FrameId};
use msg::constellation_msg::StateId;
use servo_url::ServoUrl;

#[test]
fn test_entry_replacement() {
    PipelineNamespace::install(PipelineNamespaceId(0));
    let pipeline_id = PipelineId::new();
    let frame_id = FrameId::new();
    let url = ServoUrl::parse("about:blank").expect("Infallible");
    let mut frame = Frame::new(frame_id, pipeline_id, url.clone());

    frame.prev.push(frame.current.clone());

    let new_pipeline_id = PipelineId::new();
    // Unlink this entry from the previous entry
    frame.current.replace_pipeline(new_pipeline_id, url);

    assert_eq!(frame.prev.len(), 1);
    let prev_entry = frame.prev.pop().expect("No previous entry!");
    assert_ne!(prev_entry.pipeline_id(), frame.current.pipeline_id());
}

#[test]
fn test_entry_update() {
    PipelineNamespace::install(PipelineNamespaceId(0));
    let pipeline_id = PipelineId::new();
    let frame_id = FrameId::new();
    FrameId::install(frame_id);
    let url = ServoUrl::parse("about:blank").expect("Infallible");
    let mut frame = Frame::new(frame_id, pipeline_id, url);

    // A clone will link the two entry's pipelines
    frame.prev.push(frame.current.clone());

    let new_pipeline_id = PipelineId::new();
    frame.current.update_pipeline(new_pipeline_id);
    assert_eq!(frame.pipeline_id(), new_pipeline_id);

    assert_eq!(frame.prev.len(), 1);
    let prev_entry = frame.prev.pop().expect("No previous entry!");
    assert_eq!(prev_entry.pipeline_id(), frame.current.pipeline_id());
}

#[test]
fn test_entry_discard() {
    PipelineNamespace::install(PipelineNamespaceId(0));
    let pipeline_id = PipelineId::new();
    let frame_id = FrameId::new();
    FrameId::install(frame_id);
    let url = ServoUrl::parse("about:blank").expect("Infallible");
    let mut frame = Frame::new(frame_id, pipeline_id, url.clone());

    // A clone will link the two entry's pipelines
    frame.prev.push(frame.current.clone());

    assert_eq!(frame.prev.len(), 1);
    // Cannot discard because this entry shares the same pipeline as the current entry.
    let evicted_id = frame.discard_entry(&frame.prev[0]);
    assert!(evicted_id.is_none());

    let new_pipeline_id = PipelineId::new();
    frame.current.replace_pipeline(new_pipeline_id, url);
    // Discard should work now that current is no longer linked to this entry.
    let evicted_id = frame.discard_entry(&frame.prev[0]);
    assert_eq!(evicted_id, Some(pipeline_id));
}

#[test]
fn test_state_id() {
    PipelineNamespace::install(PipelineNamespaceId(0));
    let pipeline_id = PipelineId::new();
    let frame_id = FrameId::new();
    FrameId::install(frame_id);
    let url = ServoUrl::parse("about:blank").expect("Infallible");
    let mut frame = Frame::new(frame_id, pipeline_id, url.clone());

    let state_id = StateId::new();
    frame.load_state_change(state_id, url.clone());

    assert_eq!(frame.prev.len(), 1);
    assert_eq!(frame.current.pipeline_id(), frame.prev[0].pipeline_id());

    let new_pipeline_id = PipelineId::new();
    frame.load(new_pipeline_id, url);

    let evicted_id = frame.discard_entry(&frame.prev[0]);
    assert_eq!(evicted_id, Some(pipeline_id));
    // Both entries in the session history (prev) should no longer have a pipeline id
    assert!(frame.prev[0].pipeline_id().is_none());
    assert!(frame.prev[1].pipeline_id().is_none());
    assert_eq!(frame.current.pipeline_id(), Some(new_pipeline_id));

    // Updating the pipeline id of one of the entries should update the pipeline id of
    // the other entry.
    let reloaded_pipeline_id = PipelineId::new();
    frame.prev[0].update_pipeline(reloaded_pipeline_id);
    assert_eq!(frame.prev[0].pipeline_id(), Some(reloaded_pipeline_id));
    assert_eq!(frame.prev[1].pipeline_id(), Some(reloaded_pipeline_id));
}
