/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use backtrace::Backtrace;
use constellation_msg::{HangAlert, HangAnnotation};
use constellation_msg::{MonitoredComponentId, MonitoredComponentMsg};
use ipc_channel::ipc::IpcSender;
use libc;
use servo_channel::{Receiver, Sender, base_channel, channel};
use std::collections::HashMap;
use std::sync::atomic::{AtomicBool, Ordering};
use std::time::{Duration, Instant};

/// The means of communication between monitored and monitor, inside of a "trace transaction".
pub static mut TRACE_SENDER: Option<Sender<(libc::pthread_t, Backtrace)>> = None;

lazy_static! {
    /// A flag used to create a "trace transaction" around the workflow of accessing the backtrace,
    /// from a monitored thread inside a SIGPROF handler, and the background hang monitor.
    static ref CURRENTLY_TRACING: AtomicBool = AtomicBool::new(false);
}

#[allow(unsafe_code)]
unsafe fn get_backtrace_from_monitored_component(monitored: &MonitoredComponent) -> Backtrace {
    loop {
        // Start a new "trace transaction", if none is currently ongoing.
        let currently_tracing = CURRENTLY_TRACING.compare_and_swap(
            false,
            true,
            Ordering::SeqCst,
        );
        if !currently_tracing  {
            break;
        }
    }
    // Begining of the current "trace transaction".
    let (sender, receiver) = channel();
    TRACE_SENDER = Some(sender);
    libc::pthread_kill(monitored.thread_id, libc::SIGPROF);
    let (thread_id, trace) = receiver.recv().unwrap();
    assert_eq!(thread_id, monitored.thread_id);
    TRACE_SENDER = None;
    CURRENTLY_TRACING.store(false, Ordering::SeqCst);
    // End of the current "trace transaction".
    trace
}

struct MonitoredComponent {
    thread_id: libc::pthread_t,
    last_activity: Instant,
    last_annotation: Option<HangAnnotation>,
    transient_hang_timeout: Duration,
    permanent_hang_timeout: Duration,
    sent_transient_alert: bool,
    sent_permanent_alert: bool,
    is_waiting: bool,
}

pub struct BackgroundHangMonitor {
    monitored_components: HashMap<MonitoredComponentId, MonitoredComponent>,
    constellation_chan: IpcSender<HangAlert>,
    port: Receiver<(MonitoredComponentId, MonitoredComponentMsg)>,
}

impl BackgroundHangMonitor {
    pub fn new(
        thread_id: libc::pthread_t,
        port: Receiver<(MonitoredComponentId, MonitoredComponentMsg)>,
        constellation_chan: IpcSender<HangAlert>,
        component_id: MonitoredComponentId,
        transient_hang_timeout: Duration,
        permanent_hang_timeout: Duration,
    ) -> Self {
        let mut monitor = BackgroundHangMonitor {
            monitored_components: Default::default(),
            constellation_chan,
            port,
        };
        let component = MonitoredComponent {
            thread_id,
            last_activity: Instant::now(),
            last_annotation: None,
            transient_hang_timeout,
            permanent_hang_timeout,
            sent_transient_alert: false,
            sent_permanent_alert: false,
            is_waiting: true,
        };
        assert!(
            monitor
                .monitored_components
                .insert(component_id, component)
                .is_none(),
            "This component was already registered for monitoring."
        );
        monitor
    }

    pub fn run(&mut self) -> bool {
        let received = select! {
            recv(self.port.select(), event) => {
                match event {
                    Some(msg) => Some(msg),
                    // Our sender has been dropped, quit.
                    None => return false,
                }
            },
            recv(base_channel::after(Duration::from_millis(100))) => None,
        };
        if let Some(msg) = received {
            self.handle_msg(msg);
            while let Some(another_msg) = self.port.try_recv() {
                // Handle any other incoming messages,
                // before performing a hang checkpoint.
                self.handle_msg(another_msg);
            }
        }
        self.perform_a_hang_monitor_checkpoint();
        true
    }

    fn handle_msg(&mut self, msg: (MonitoredComponentId, MonitoredComponentMsg)) {
        match msg {
            (component_id, MonitoredComponentMsg::NotifyActivity(annotation)) => {
                let mut component = self
                    .monitored_components
                    .get_mut(&component_id)
                    .expect("Receiced NotifyActivity for an unknown component");
                component.last_activity = Instant::now();
                component.last_annotation = Some(annotation);
                component.sent_transient_alert = false;
                component.sent_permanent_alert = false;
                component.is_waiting = false;
            },
            (component_id, MonitoredComponentMsg::NotifyWait) => {
                let mut component = self
                    .monitored_components
                    .get_mut(&component_id)
                    .expect("Receiced NotifyWait for an unknown component");
                component.last_activity = Instant::now();
                component.sent_transient_alert = false;
                component.sent_permanent_alert = false;
                component.is_waiting = true;
            },
        }
    }

    #[allow(unsafe_code)]
    fn perform_a_hang_monitor_checkpoint(&mut self) {
        for (component_id, mut monitored) in self.monitored_components.iter_mut() {
            if monitored.is_waiting {
                continue;
            }
            let last_annotation = monitored.last_annotation.unwrap();
            if monitored.last_activity.elapsed() > monitored.permanent_hang_timeout {
                match monitored.sent_permanent_alert {
                    true => continue,
                    false => {
                        let trace = unsafe {
                            get_backtrace_from_monitored_component(&monitored)
                        };
                        let _ = self
                            .constellation_chan
                            .send(
                                HangAlert::Permanent(
                                    component_id.clone(),
                                    last_annotation,
                                    format!("{:?}", trace)
                                )
                            );
                        monitored.sent_permanent_alert = true;
                        continue;
                    },
                }
            }
            if monitored.last_activity.elapsed() > monitored.transient_hang_timeout {
                match monitored.sent_transient_alert {
                    true => continue,
                    false => {
                        let _ = self
                            .constellation_chan
                            .send(HangAlert::Transient(component_id.clone(), last_annotation));
                        monitored.sent_transient_alert = true;
                    },
                }
                continue;
            }
        }
    }
}
