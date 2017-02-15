/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use constellation::Frame;
use msg::constellation_msg::{PipelineId, PipelineNamespace, PipelineNamespaceId, FrameId};
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
