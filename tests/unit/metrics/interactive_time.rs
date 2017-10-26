/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use ipc_channel::ipc;
use metrics::{InteractiveMetrics, InteractiveFlag, InteractiveWindow};
use metrics::{ProfilerMetadataFactory, ProgressiveWebMetric};
use profile_traits::time::{ProfilerChan, TimerMetadata};
use time;

struct DummyProfilerMetadataFactory {}
impl ProfilerMetadataFactory for DummyProfilerMetadataFactory {
    fn new_metadata(&self) -> Option<TimerMetadata> {
        None
    }
}


fn test_interactive() -> InteractiveMetrics {
    let (sender, _) = ipc::channel().unwrap();
    let profiler_chan = ProfilerChan(sender);
    let mut interactive = InteractiveMetrics::new(profiler_chan);

    assert_eq!((&interactive).get_navigation_start(), None);
    assert_eq!(interactive.get_tti(), None);

    interactive.set_navigation_start(time::precise_time_ns() as f64);

    interactive
}

#[test]
fn test_set_dcl() {
    let profiler_metadata_factory = DummyProfilerMetadataFactory {};

    let interactive = test_interactive();
    interactive.maybe_set_tti(&profiler_metadata_factory, InteractiveFlag::DOMContentLoaded);
    let dcl = interactive.get_dom_content_loaded();
    assert!(dcl.is_some());

    //try to overwrite
    interactive.maybe_set_tti(&profiler_metadata_factory, InteractiveFlag::DOMContentLoaded);
    assert_eq!(interactive.get_dom_content_loaded(), dcl);
    assert_eq!(interactive.get_tti(), None);
}

#[test]
fn test_set_mta() {
    let profiler_metadata_factory = DummyProfilerMetadataFactory {};

    let interactive = test_interactive();
    let t = time::precise_time_ns();
    interactive.maybe_set_tti(
        &profiler_metadata_factory,
        InteractiveFlag::TimeToInteractive(t as f64),
    );
    let mta = interactive.get_main_thread_available();
    assert!(mta.is_some());
    assert_eq!(mta, Some(t as f64));

    //try to overwrite
    interactive.maybe_set_tti(
        &profiler_metadata_factory,
        InteractiveFlag::TimeToInteractive(time::precise_time_ns() as f64),
    );
    assert_eq!(interactive.get_main_thread_available(), mta);
    assert_eq!(interactive.get_tti(), None);
}

#[test]
fn test_set_tti_dcl() {
    let profiler_metadata_factory = DummyProfilerMetadataFactory {};

    let interactive = test_interactive();
    let t = time::precise_time_ns();
    interactive.maybe_set_tti(
        &profiler_metadata_factory,
        InteractiveFlag::TimeToInteractive(t as f64),
    );
    let mta = interactive.get_main_thread_available();
    assert!(mta.is_some());

    interactive.maybe_set_tti(&profiler_metadata_factory, InteractiveFlag::DOMContentLoaded);
    let dcl = interactive.get_dom_content_loaded();
    assert!(dcl.is_some());

    let interactive_time = dcl.unwrap() - (&interactive).get_navigation_start().unwrap();
    assert_eq!(interactive.get_tti(), Some(interactive_time));
}

#[test]
fn test_set_tti_mta() {
    let profiler_metadata_factory = DummyProfilerMetadataFactory {};

    let interactive = test_interactive();
    interactive.maybe_set_tti(&profiler_metadata_factory, InteractiveFlag::DOMContentLoaded);
    let dcl = interactive.get_dom_content_loaded();
    assert!(dcl.is_some());

    let t = time::precise_time_ns();
    interactive.maybe_set_tti(
        &profiler_metadata_factory,
        InteractiveFlag::TimeToInteractive(t as f64),
    );
    let mta = interactive.get_main_thread_available();
    assert!(mta.is_some());

    let interactive_time = mta.unwrap() - (&interactive).get_navigation_start().unwrap();
    assert_eq!(interactive.get_tti(), Some(interactive_time));
}

// TODO InteractiveWindow tests
