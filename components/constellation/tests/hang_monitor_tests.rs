/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

extern crate constellation;
extern crate msg;

use constellation::background_hang_monitor::{BackgroundHangMonitor, HangAlert};
use msg::constellation_msg::{MonitoredComponentType, MonitoredComponentMsg, PipelineId};
use msg::constellation_msg::{PipelineNamespace, PipelineNamespaceId};
use std::thread;
use std::time::Duration;


#[test]
fn test_hang_monitoring() {
    PipelineNamespace::install(PipelineNamespaceId(3));
    let mut monitor: BackgroundHangMonitor = Default::default();
    let component_type = MonitoredComponentType::Script(PipelineId::new());
    let component_type_2 = MonitoredComponentType::Script(PipelineId::new());
    let task_timeout = Duration::from_millis(1);
    let max_timeout = Duration::from_millis(5);
    // Register a first component that will be handling tasks.
    let msg = MonitoredComponentMsg::RegisterComponent(component_type.clone(), task_timeout, max_timeout);
    monitor.handle_msg(msg);
    // Register a second component that will not handle any tasks, and also not generate any hang alerts,
    // since registration sets it into "waiting" state.
    let msg = MonitoredComponentMsg::RegisterComponent(component_type_2, task_timeout, max_timeout);
    monitor.handle_msg(msg);
    let msg = MonitoredComponentMsg::NotifyActivity(component_type.clone());
    monitor.handle_msg(msg);

    // Sleep until the "task-timeout" has been reached.
    thread::sleep(Duration::from_millis(1));

    // Check for a transient hang alert.
    monitor.perform_a_hang_monitor_checkpoint();
    let mut alerts = monitor.collect_hang_alerts();
    assert_eq!(alerts.len(), 1);
    match alerts.pop_front().unwrap() {
        HangAlert::TransientHang(component) => {
            assert_eq!(component, component_type)
        },
        _ => unreachable!()
    }

    // Sleep until the "max-timeout" has been reached.
    thread::sleep(Duration::from_millis(5));

    // Nothing happens unless we call perform_a_hang_monitor_checkpoint
    let alerts = monitor.collect_hang_alerts();
    assert_eq!(alerts.len(), 0);

    // Check for a permanent hang alert.
    monitor.perform_a_hang_monitor_checkpoint();
    let mut alerts = monitor.collect_hang_alerts();
    assert_eq!(alerts.len(), 1);
    match alerts.pop_front().unwrap() {
        HangAlert::PermanentHang(component) => {
            assert_eq!(component, component_type)
        },
        _ => unreachable!()
    }

    // Performing another checkpoint gives us the same alert again.
    monitor.perform_a_hang_monitor_checkpoint();
    let mut alerts = monitor.collect_hang_alerts();
    assert_eq!(alerts.len(), 1);
    match alerts.pop_front().unwrap() {
        HangAlert::PermanentHang(component) => {
            assert_eq!(component, component_type)
        },
        _ => unreachable!()
    }

    // Now the component is not hanging anymore.
    let msg = MonitoredComponentMsg::NotifyActivity(component_type.clone());
    monitor.handle_msg(msg);
    monitor.perform_a_hang_monitor_checkpoint();
    let alerts = monitor.collect_hang_alerts();
    assert_eq!(alerts.len(), 0);

    // Now the component is waiting for a new task.
    let msg = MonitoredComponentMsg::NotifyWait(component_type.clone());
    monitor.handle_msg(msg);

    // Sleep for a while.
    thread::sleep(Duration::from_millis(6));

    // The component is still waiting, but not hanging.
    monitor.perform_a_hang_monitor_checkpoint();
    let alerts = monitor.collect_hang_alerts();
    assert_eq!(alerts.len(), 0);

    // New task handling starts.
    let msg = MonitoredComponentMsg::NotifyActivity(component_type.clone());
    monitor.handle_msg(msg);

    // Sleep for a while.
    thread::sleep(Duration::from_millis(1));

    // We're getting new hang alerts for the latest task.
    monitor.perform_a_hang_monitor_checkpoint();
    let mut alerts = monitor.collect_hang_alerts();
    assert_eq!(alerts.len(), 1);
    match alerts.pop_front().unwrap() {
        HangAlert::TransientHang(component) => {
            assert_eq!(component, component_type)
        },
        _ => unreachable!()
    }
}
