/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

#[cfg(any(target_os = "android", target_os = "linux"))]
use crate::sampler::install_sigprof_handler;
use crate::sampler::{get_thread_id, suspend_and_sample_thread, MonitoredThreadId};
use ipc_channel::ipc::IpcSender;
use msg::constellation_msg::MonitoredComponentId;
use msg::constellation_msg::{
    BackgroundHangMonitor, BackgroundHangMonitorClone, BackgroundHangMonitorRegister,
};
use msg::constellation_msg::{HangAlert, HangAnnotation};
use servo_channel::{base_channel, channel, Receiver, Sender};
use std::cell::Cell;
use std::collections::HashMap;
use std::thread;
use std::time::{Duration, Instant};

#[derive(Clone)]
pub struct HangMonitorRegister {
    sender: Sender<(MonitoredComponentId, MonitoredComponentMsg)>,
}

impl HangMonitorRegister {
    /// Start a new hang monitor worker, and return a handle to register components for monitoring.
    pub fn init(constellation_chan: IpcSender<HangAlert>) -> Box<BackgroundHangMonitorRegister> {
        let (sender, port) = channel();
        let _ = thread::Builder::new().spawn(move || {
            let mut monitor = { BackgroundHangMonitorWorker::new(constellation_chan, port) };
            while monitor.run() {
                // Monitoring until all senders have been dropped...
            }
        });
        Box::new(HangMonitorRegister { sender })
    }
}

impl BackgroundHangMonitorRegister for HangMonitorRegister {
    /// Register a component for monitoring.
    /// Returns a dedicated wrapper around a sender
    /// to be used for communication with the hang monitor worker.
    #[allow(unsafe_code)]
    fn register_component(
        &self,
        component_id: MonitoredComponentId,
        transient_hang_timeout: Duration,
        permanent_hang_timeout: Duration,
    ) -> Box<BackgroundHangMonitor> {
        let bhm_chan = BackgroundHangMonitorChan::new(self.sender.clone(), component_id);
        let thread_id = unsafe {
            #[cfg(any(target_os = "android", target_os = "linux"))]
            install_sigprof_handler();
            get_thread_id()
        };
        bhm_chan.send(MonitoredComponentMsg::Register(
            thread_id,
            transient_hang_timeout,
            permanent_hang_timeout,
        ));
        Box::new(bhm_chan)
    }
}

impl BackgroundHangMonitorClone for HangMonitorRegister {
    fn clone_box(&self) -> Box<BackgroundHangMonitorRegister> {
        Box::new(self.clone())
    }
}

/// Messages sent from monitored components to the monitor.
pub enum MonitoredComponentMsg {
    /// Register component for monitoring,
    Register(MonitoredThreadId, Duration, Duration),
    /// Notify start of new activity for a given component,
    NotifyActivity(HangAnnotation),
    /// Notify start of waiting for a new task to come-in for processing.
    NotifyWait,
}

/// A wrapper around a sender to the monitor,
/// which will send the Id of the monitored component along with each message,
/// and keep track of whether the monitor is still listening on the other end.
pub struct BackgroundHangMonitorChan {
    sender: Sender<(MonitoredComponentId, MonitoredComponentMsg)>,
    component_id: MonitoredComponentId,
    disconnected: Cell<bool>,
}

impl BackgroundHangMonitorChan {
    pub fn new(
        sender: Sender<(MonitoredComponentId, MonitoredComponentMsg)>,
        component_id: MonitoredComponentId,
    ) -> Self {
        BackgroundHangMonitorChan {
            sender,
            component_id: component_id,
            disconnected: Default::default(),
        }
    }

    pub fn send(&self, msg: MonitoredComponentMsg) {
        if self.disconnected.get() {
            return;
        }
        if let Err(_) = self.sender.send((self.component_id.clone(), msg)) {
            warn!("BackgroundHangMonitor has gone away");
            self.disconnected.set(true);
        }
    }
}

impl BackgroundHangMonitor for BackgroundHangMonitorChan {
    fn notify_activity(&self, annotation: HangAnnotation) {
        let msg = MonitoredComponentMsg::NotifyActivity(annotation);
        self.send(msg);
    }
    fn notify_wait(&self) {
        let msg = MonitoredComponentMsg::NotifyWait;
        self.send(msg);
    }
}

struct MonitoredComponent {
    thread_id: MonitoredThreadId,
    last_activity: Instant,
    last_annotation: Option<HangAnnotation>,
    transient_hang_timeout: Duration,
    permanent_hang_timeout: Duration,
    sent_transient_alert: bool,
    sent_permanent_alert: bool,
    is_waiting: bool,
}

pub struct BackgroundHangMonitorWorker {
    monitored_components: HashMap<MonitoredComponentId, MonitoredComponent>,
    constellation_chan: IpcSender<HangAlert>,
    port: Receiver<(MonitoredComponentId, MonitoredComponentMsg)>,
}

impl BackgroundHangMonitorWorker {
    pub fn new(
        constellation_chan: IpcSender<HangAlert>,
        port: Receiver<(MonitoredComponentId, MonitoredComponentMsg)>,
    ) -> Self {
        Self {
            monitored_components: Default::default(),
            constellation_chan,
            port,
        }
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
            (
                component_id,
                MonitoredComponentMsg::Register(
                    thread_id,
                    transient_hang_timeout,
                    permanent_hang_timeout,
                ),
            ) => {
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
                    self.monitored_components
                        .insert(component_id, component)
                        .is_none(),
                    "This component was already registered for monitoring."
                );
            },
            (component_id, MonitoredComponentMsg::NotifyActivity(annotation)) => {
                let component = self
                    .monitored_components
                    .get_mut(&component_id)
                    .expect("Received NotifyActivity for an unknown component");
                component.last_activity = Instant::now();
                component.last_annotation = Some(annotation);
                component.sent_transient_alert = false;
                component.sent_permanent_alert = false;
                component.is_waiting = false;
            },
            (component_id, MonitoredComponentMsg::NotifyWait) => {
                let component = self
                    .monitored_components
                    .get_mut(&component_id)
                    .expect("Received NotifyWait for an unknown component");
                component.last_activity = Instant::now();
                component.sent_transient_alert = false;
                component.sent_permanent_alert = false;
                component.is_waiting = true;
            },
        }
    }

    #[allow(unsafe_code)]
    fn perform_a_hang_monitor_checkpoint(&mut self) {
        for (component_id, monitored) in self.monitored_components.iter_mut() {
            if monitored.is_waiting {
                continue;
            }
            let last_annotation = monitored.last_annotation.unwrap();
            if monitored.last_activity.elapsed() > monitored.permanent_hang_timeout {
                if monitored.sent_permanent_alert {
                    continue;
                }
                let profile = unsafe { suspend_and_sample_thread(monitored.thread_id) };
                let _ = self.constellation_chan.send(HangAlert::Permanent(
                    component_id.clone(),
                    last_annotation,
                    profile,
                ));
                monitored.sent_permanent_alert = true;
                continue;
            }
            if monitored.last_activity.elapsed() > monitored.transient_hang_timeout {
                if monitored.sent_transient_alert {
                    continue;
                }
                let _ = self
                    .constellation_chan
                    .send(HangAlert::Transient(component_id.clone(), last_annotation));
                monitored.sent_transient_alert = true;
            }
        }
    }
}
