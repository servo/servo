/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

#![allow(unused_imports)]

use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::Duration;

use background_hang_monitor::HangMonitorRegister;
use background_hang_monitor_api::{
    BackgroundHangMonitorControlMsg, BackgroundHangMonitorExitSignal, HangAlert, HangAnnotation,
    HangMonitorAlert, MonitoredComponentId, MonitoredComponentType, ScriptHangAnnotation,
};
use base::id::TEST_PIPELINE_ID;
use ipc_channel::ipc;

lazy_static::lazy_static! {
    static ref SERIAL: Mutex<()> = Mutex::new(());
}

#[test]
fn test_hang_monitoring() {
    let _lock = SERIAL.lock().unwrap();

    let (background_hang_monitor_ipc_sender, background_hang_monitor_receiver) =
        ipc::channel().expect("ipc channel failure");
    let (_sampler_sender, sampler_receiver) = ipc::channel().expect("ipc channel failure");

    let background_hang_monitor_register = HangMonitorRegister::init(
        background_hang_monitor_ipc_sender.clone(),
        sampler_receiver,
        true,
    );
    let background_hang_monitor = background_hang_monitor_register.register_component(
        MonitoredComponentId(TEST_PIPELINE_ID, MonitoredComponentType::Script),
        Duration::from_millis(10),
        Duration::from_millis(1000),
        None,
    );

    // Start an activity.
    let hang_annotation = HangAnnotation::Script(ScriptHangAnnotation::AttachLayout);
    background_hang_monitor.notify_activity(hang_annotation);

    // Sleep until the "transient" timeout has been reached.
    thread::sleep(Duration::from_millis(10));

    // Check for a transient hang alert.
    match background_hang_monitor_receiver.recv().unwrap() {
        HangMonitorAlert::Hang(HangAlert::Transient(component_id, _annotation)) => {
            let expected = MonitoredComponentId(TEST_PIPELINE_ID, MonitoredComponentType::Script);
            assert_eq!(expected, component_id);
        },
        _ => unreachable!(),
    }

    // Sleep until the "permanent" timeout has been reached.
    thread::sleep(Duration::from_millis(1000));

    // Check for a permanent hang alert.
    match background_hang_monitor_receiver.recv().unwrap() {
        HangMonitorAlert::Hang(HangAlert::Permanent(component_id, _annotation, _profile)) => {
            let expected = MonitoredComponentId(TEST_PIPELINE_ID, MonitoredComponentType::Script);
            assert_eq!(expected, component_id);
        },
        _ => unreachable!(),
    }

    // Now the component is not hanging anymore.
    background_hang_monitor.notify_activity(hang_annotation);
    assert!(background_hang_monitor_receiver.try_recv().is_err());

    // Sleep for a while.
    thread::sleep(Duration::from_millis(10));

    // Check for a transient hang alert.
    match background_hang_monitor_receiver.recv().unwrap() {
        HangMonitorAlert::Hang(HangAlert::Transient(component_id, _annotation)) => {
            let expected = MonitoredComponentId(TEST_PIPELINE_ID, MonitoredComponentType::Script);
            assert_eq!(expected, component_id);
        },
        _ => unreachable!(),
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
        HangMonitorAlert::Hang(HangAlert::Transient(component_id, _annotation)) => {
            let expected = MonitoredComponentId(TEST_PIPELINE_ID, MonitoredComponentType::Script);
            assert_eq!(expected, component_id);
        },
        _ => unreachable!(),
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
// https://github.com/servo/servo/issues/28270
#[cfg(not(any(target_os = "windows", target_os = "macos")))]
fn test_hang_monitoring_unregister() {
    let _lock = SERIAL.lock().unwrap();

    let (background_hang_monitor_ipc_sender, background_hang_monitor_receiver) =
        ipc::channel().expect("ipc channel failure");
    let (_sampler_sender, sampler_receiver) = ipc::channel().expect("ipc channel failure");

    let background_hang_monitor_register = HangMonitorRegister::init(
        background_hang_monitor_ipc_sender.clone(),
        sampler_receiver,
        true,
    );
    let background_hang_monitor = background_hang_monitor_register.register_component(
        MonitoredComponentId(TEST_PIPELINE_ID, MonitoredComponentType::Script),
        Duration::from_millis(10),
        Duration::from_millis(1000),
        None,
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

// Perform two certain steps in `test_hang_monitoring_exit_signal_inner` in
// different orders to check for the race condition that
// caused <https://github.com/servo/servo/issues/28270> and
// <https://github.com/servo/servo/issues/27191>.
#[test]
fn test_hang_monitoring_exit_signal1() {
    test_hang_monitoring_exit_signal_inner(|e1, e2| {
        e1();
        thread::sleep(Duration::from_millis(100));
        e2();
    });
}

#[test]
fn test_hang_monitoring_exit_signal2() {
    test_hang_monitoring_exit_signal_inner(|e1, e2| {
        e1();
        e2();
    });
}

#[test]
fn test_hang_monitoring_exit_signal3() {
    test_hang_monitoring_exit_signal_inner(|e1, e2| {
        e2();
        e1();
    });
}

#[test]
fn test_hang_monitoring_exit_signal4() {
    test_hang_monitoring_exit_signal_inner(|e1, e2| {
        e2();
        thread::sleep(Duration::from_millis(100));
        e1();
    });
}

fn test_hang_monitoring_exit_signal_inner(op_order: fn(&mut dyn FnMut(), &mut dyn FnMut())) {
    let _lock = SERIAL.lock().unwrap();

    let (background_hang_monitor_ipc_sender, _background_hang_monitor_receiver) =
        ipc::channel().expect("ipc channel failure");
    let (control_sender, control_receiver) = ipc::channel().expect("ipc channel failure");

    struct BHMExitSignal {
        closing: Arc<AtomicBool>,
    }

    impl BackgroundHangMonitorExitSignal for BHMExitSignal {
        fn signal_to_exit(&self) {
            self.closing.store(true, Ordering::SeqCst);
        }
    }

    let closing = Arc::new(AtomicBool::new(false));
    let mut signal = Some(Box::new(BHMExitSignal {
        closing: closing.clone(),
    }));

    // Init a worker, without active monitoring.
    let background_hang_monitor_register = HangMonitorRegister::init(
        background_hang_monitor_ipc_sender.clone(),
        control_receiver,
        false,
    );

    let mut background_hang_monitor = None;
    let (exit_sender, exit_receiver) = ipc::channel().expect("Failed to create IPC channel!");
    let mut exit_sender = Some(exit_sender);

    // `op_order` determines the order in which these two closures are
    // executed.
    op_order(
        &mut || {
            // Register a component.
            background_hang_monitor = Some(background_hang_monitor_register.register_component(
                MonitoredComponentId(TEST_PIPELINE_ID, MonitoredComponentType::Script),
                Duration::from_millis(10),
                Duration::from_millis(1000),
                Some(signal.take().unwrap()),
            ));
        },
        &mut || {
            // Send the exit message.
            control_sender
                .send(BackgroundHangMonitorControlMsg::Exit(
                    exit_sender.take().unwrap(),
                ))
                .unwrap();
        },
    );

    // Assert we receive a confirmation back.
    assert!(exit_receiver.recv().is_ok());

    // Assert we get the exit signal.
    while !closing.load(Ordering::SeqCst) {
        thread::sleep(Duration::from_millis(10));
    }
}
