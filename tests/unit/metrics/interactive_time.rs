/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use base::cross_process_instant::CrossProcessInstant;
use ipc_channel::ipc;
use metrics::{InteractiveFlag, InteractiveMetrics, ProfilerMetadataFactory, ProgressiveWebMetric};
use profile_traits::time::{ProfilerChan, TimerMetadata};
use servo_url::ServoUrl;

struct DummyProfilerMetadataFactory {}
impl ProfilerMetadataFactory for DummyProfilerMetadataFactory {
    fn new_metadata(&self) -> Option<TimerMetadata> {
        None
    }
}

fn test_interactive() -> InteractiveMetrics {
    let (sender, _) = ipc::channel().unwrap();
    let profiler_chan = ProfilerChan(sender);
    let mut interactive =
        InteractiveMetrics::new(profiler_chan, ServoUrl::parse("about:blank").unwrap());

    assert_eq!((&interactive).get_navigation_start(), None);
    assert_eq!(interactive.get_tti(), None);

    interactive.set_navigation_start(CrossProcessInstant::now());

    interactive
}

#[test]
fn test_set_dcl() {
    let profiler_metadata_factory = DummyProfilerMetadataFactory {};

    let interactive = test_interactive();
    interactive.maybe_set_tti(
        &profiler_metadata_factory,
        InteractiveFlag::DOMContentLoaded,
    );
    let dcl = interactive.get_dom_content_loaded();
    assert!(dcl.is_some());

    //try to overwrite
    interactive.maybe_set_tti(
        &profiler_metadata_factory,
        InteractiveFlag::DOMContentLoaded,
    );
    assert_eq!(interactive.get_dom_content_loaded(), dcl);
    assert_eq!(interactive.get_tti(), None);
}

#[test]
fn test_set_mta() {
    let profiler_metadata_factory = DummyProfilerMetadataFactory {};

    let interactive = test_interactive();
    let now = CrossProcessInstant::now();
    interactive.maybe_set_tti(
        &profiler_metadata_factory,
        InteractiveFlag::TimeToInteractive(now),
    );
    let main_thread_available_time = interactive.get_main_thread_available();
    assert!(main_thread_available_time.is_some());
    assert_eq!(main_thread_available_time, Some(now));

    //try to overwrite
    interactive.maybe_set_tti(
        &profiler_metadata_factory,
        InteractiveFlag::TimeToInteractive(CrossProcessInstant::now()),
    );
    assert_eq!(
        interactive.get_main_thread_available(),
        main_thread_available_time
    );
    assert_eq!(interactive.get_tti(), None);
}

#[test]
fn test_set_tti_dcl() {
    let profiler_metadata_factory = DummyProfilerMetadataFactory {};

    let interactive = test_interactive();
    let now = CrossProcessInstant::now();
    interactive.maybe_set_tti(
        &profiler_metadata_factory,
        InteractiveFlag::TimeToInteractive(now),
    );
    let main_thread_available_time = interactive.get_main_thread_available();
    assert!(main_thread_available_time.is_some());

    interactive.maybe_set_tti(
        &profiler_metadata_factory,
        InteractiveFlag::DOMContentLoaded,
    );
    let dom_content_loaded_time = interactive.get_dom_content_loaded();
    assert!(dom_content_loaded_time.is_some());

    assert_eq!(interactive.get_tti(), dom_content_loaded_time);
}

#[test]
fn test_set_tti_mta() {
    let profiler_metadata_factory = DummyProfilerMetadataFactory {};

    let interactive = test_interactive();
    interactive.maybe_set_tti(
        &profiler_metadata_factory,
        InteractiveFlag::DOMContentLoaded,
    );
    let dcl = interactive.get_dom_content_loaded();
    assert!(dcl.is_some());

    let time = CrossProcessInstant::now();
    interactive.maybe_set_tti(
        &profiler_metadata_factory,
        InteractiveFlag::TimeToInteractive(time),
    );
    let mta = interactive.get_main_thread_available();
    assert!(mta.is_some());

    assert_eq!(interactive.get_tti(), mta);
}

// TODO InteractiveWindow tests
