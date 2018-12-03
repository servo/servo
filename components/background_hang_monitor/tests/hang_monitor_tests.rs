/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

#[macro_use]
extern crate lazy_static;

use background_hang_monitor::HangMonitorRegister;
use ipc_channel::ipc;
use msg::constellation_msg::ScriptHangAnnotation;
use msg::constellation_msg::TEST_PIPELINE_ID;
use msg::constellation_msg::{HangAlert, HangAnnotation};
use msg::constellation_msg::{MonitoredComponentId, MonitoredComponentType};
use std::sync::Mutex;
use std::thread;
use std::time::Duration;

lazy_static! {
    static ref SERIAL: Mutex<()> = Mutex::new(());
}

#[test]
fn test_hang_monitoring() {
    let _lock = SERIAL.lock().unwrap();

    let (background_hang_monitor_ipc_sender, background_hang_monitor_receiver) =
        ipc::channel().expect("ipc channel failure");

    let background_hang_monitor_register =
        HangMonitorRegister::init(background_hang_monitor_ipc_sender.clone());
    let background_hang_monitor = background_hang_monitor_register.register_component(
        MonitoredComponentId(TEST_PIPELINE_ID, MonitoredComponentType::Script),
        Duration::from_millis(10),
        Duration::from_millis(1000),
    );

    // Start an activity.
    let hang_annotation = HangAnnotation::Script(ScriptHangAnnotation::AttachLayout);
    background_hang_monitor.notify_activity(hang_annotation);

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
        HangAlert::Permanent(component_id, _annotation, _profile) => {
            let expected = MonitoredComponentId(TEST_PIPELINE_ID, MonitoredComponentType::Script);
            assert_eq!(expected, component_id);
        },
        HangAlert::Transient(..) => unreachable!(),
    }

    // Now the component is not hanging anymore.
    background_hang_monitor.notify_activity(hang_annotation);
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
    background_hang_monitor.notify_wait();

    // Sleep for a while.
    thread::sleep(Duration::from_millis(100));

    // The component is still waiting, but not hanging.
    assert!(background_hang_monitor_receiver.try_recv().is_err());

    // New task handling starts.
    background_hang_monitor.notify_activity(hang_annotation);

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
    drop(background_hang_monitor_register);
    drop(background_hang_monitor);

    // Sleep until the "max-timeout" has been reached.
    thread::sleep(Duration::from_millis(1000));

    // Still no new alerts because the hang monitor has shut-down already.
    assert!(background_hang_monitor_receiver.try_recv().is_err());
}

#[test]
fn test_hang_monitoring_unregister() {
    let _lock = SERIAL.lock().unwrap();

    let (background_hang_monitor_ipc_sender, background_hang_monitor_receiver) =
        ipc::channel().expect("ipc channel failure");

    let background_hang_monitor_register =
        HangMonitorRegister::init(background_hang_monitor_ipc_sender.clone());
    let background_hang_monitor = background_hang_monitor_register.register_component(
        MonitoredComponentId(TEST_PIPELINE_ID, MonitoredComponentType::Script),
        Duration::from_millis(10),
        Duration::from_millis(1000),
    );

    // Start an activity.
    let hang_annotation = HangAnnotation::Script(ScriptHangAnnotation::AttachLayout);
    background_hang_monitor.notify_activity(hang_annotation);

    // Unregister the component.
    background_hang_monitor.unregister();

    // Sleep until the "transient" timeout has been reached.
    thread::sleep(Duration::from_millis(10));

    // No new alert yet
    assert!(background_hang_monitor_receiver.try_recv().is_err());
}
