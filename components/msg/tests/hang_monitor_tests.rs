/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

extern crate ipc_channel;
extern crate msg;

use ipc_channel::ipc;
use msg::constellation_msg::{HangAnnotation, HangAlert, ScriptHangAnnotation, init_background_hang_monitor};
use msg::constellation_msg::{MonitoredComponentType, MonitoredComponentMsg, MonitoredComponentId};
use msg::constellation_msg::TEST_PIPELINE_ID;
use std::sync::{Arc, Barrier};
use std::thread;
use std::time::Duration;

#[test]
fn test_hang_monitoring() {
    let (background_hang_monitor_sender, background_hang_monitor_receiver) =
        ipc::channel().expect("ipc channel failure");
    let background_hang_monitor_chan = init_background_hang_monitor(
        TEST_PIPELINE_ID,
        MonitoredComponentType::Script,
        background_hang_monitor_sender.clone(),
        Duration::from_millis(10),
        Duration::from_millis(1000),
    );

    // Start several monitors in several threads that will hang,
    // to check there is no crash
    // when two monitor try to do a "trace transaction" at (approx) the same time.
    let barrier = Arc::new(Barrier::new(5));
    for _ in 0..5 {
        let c = barrier.clone();
        let (background_hang_monitor_sender, _background_hang_monitor_receiver) =
            ipc::channel().expect("ipc channel failure");
        let _ = thread::Builder::new().spawn(move || {
            let sender = init_background_hang_monitor(
                TEST_PIPELINE_ID,
                MonitoredComponentType::Script,
                background_hang_monitor_sender,
                Duration::from_millis(10),
                Duration::from_millis(1000),
            );
            c.wait();
            let hang_annotation = ScriptHangAnnotation::AttachLayout;
            let msg = MonitoredComponentMsg::NotifyActivity(
                HangAnnotation::Script(hang_annotation),
            );
            sender.send(msg);
            thread::sleep(Duration::from_millis(1500));
        });
    }

    // Start a first activity.
    let hang_annotation = ScriptHangAnnotation::AttachLayout;
    let msg = MonitoredComponentMsg::NotifyActivity(
        HangAnnotation::Script(hang_annotation),
    );
    background_hang_monitor_chan.send(msg);

    // Sleep until the "transient" timeout has been reached.
    thread::sleep(Duration::from_millis(10));

    // Check for a transient hang alert.
    match background_hang_monitor_receiver.recv().unwrap() {
        HangAlert::Transient(component_id, _annotation) => {
            let expected = MonitoredComponentId(TEST_PIPELINE_ID, MonitoredComponentType::Script);
            assert_eq!(expected, component_id);
        },
        HangAlert::Permanent(..) => unreachable!(),
    }

    // Sleep until the "permanent" timeout has been reached.
    thread::sleep(Duration::from_millis(1000));

    // Check for a permanent hang alert.
    match background_hang_monitor_receiver.recv().unwrap() {
        HangAlert::Permanent(component_id, _annotation, _trace) => {
            let expected = MonitoredComponentId(TEST_PIPELINE_ID, MonitoredComponentType::Script);
            assert_eq!(expected, component_id);
        },
        HangAlert::Transient(..) => unreachable!(),
    }

    // Now the component is not hanging anymore.
    let msg = MonitoredComponentMsg::NotifyActivity(
        HangAnnotation::Script(hang_annotation),
    );
    background_hang_monitor_chan.send(msg);
    assert!(background_hang_monitor_receiver.try_recv().is_err());

    // Sleep for a while.
    thread::sleep(Duration::from_millis(10));

    // Check for a transient hang alert.
    match background_hang_monitor_receiver.recv().unwrap() {
        HangAlert::Transient(component_id, _annotation) => {
            let expected = MonitoredComponentId(TEST_PIPELINE_ID, MonitoredComponentType::Script);
            assert_eq!(expected, component_id);
        },
        HangAlert::Permanent(..) => unreachable!(),
    }

    // Now the component is waiting for a new task.
    let msg = MonitoredComponentMsg::NotifyWait;
    background_hang_monitor_chan.send(msg);

    // Sleep for a while.
    thread::sleep(Duration::from_millis(100));

    // The component is still waiting, but not hanging.
    assert!(background_hang_monitor_receiver.try_recv().is_err());

    // New task handling starts.
    let msg = MonitoredComponentMsg::NotifyActivity(
        HangAnnotation::Script(hang_annotation),
    );
    background_hang_monitor_chan.send(msg);

    // Sleep for a while.
    thread::sleep(Duration::from_millis(10));

    // We're getting new hang alerts for the latest task.
    match background_hang_monitor_receiver.recv().unwrap() {
        HangAlert::Transient(component_id, _annotation) => {
            let expected = MonitoredComponentId(TEST_PIPELINE_ID, MonitoredComponentType::Script);
            assert_eq!(expected, component_id);
        },
        HangAlert::Permanent(..) => unreachable!(),
    }

    // No new alert yet
    assert!(background_hang_monitor_receiver.try_recv().is_err());

    // Shut-down the hang monitor
    drop(background_hang_monitor_chan);

    // Sleep until the "max-timeout" has been reached.
    thread::sleep(Duration::from_millis(1000));

    // Still no new alerts because the hang monitor has shut-down already.
    assert!(background_hang_monitor_receiver.try_recv().is_err());
}
