/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use crate::sampler::{NativeStack, Sampler};
use crossbeam_channel::{after, unbounded, Receiver, Sender};
use ipc_channel::ipc::{IpcReceiver, IpcSender};
use ipc_channel::router::ROUTER;
use msg::constellation_msg::MonitoredComponentId;
use msg::constellation_msg::{
    BackgroundHangMonitor, BackgroundHangMonitorClone, BackgroundHangMonitorRegister,
};
use msg::constellation_msg::{HangAlert, HangAnnotation, HangMonitorAlert, SamplerControlMsg};
use std::cell::Cell;
use std::collections::{HashMap, VecDeque};
use std::thread;
use std::time::{Duration, Instant};

#[derive(Clone)]
pub struct HangMonitorRegister {
    sender: Sender<(MonitoredComponentId, MonitoredComponentMsg)>,
}

impl HangMonitorRegister {
    /// Start a new hang monitor worker, and return a handle to register components for monitoring.
    pub fn init(
        constellation_chan: IpcSender<HangMonitorAlert>,
        control_port: IpcReceiver<SamplerControlMsg>,
    ) -> Box<BackgroundHangMonitorRegister> {
        let (sender, port) = unbounded();
        let _ = thread::Builder::new().spawn(move || {
            let mut monitor =
                BackgroundHangMonitorWorker::new(constellation_chan, control_port, port);
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
    fn register_component(
        &self,
        component_id: MonitoredComponentId,
        transient_hang_timeout: Duration,
        permanent_hang_timeout: Duration,
    ) -> Box<BackgroundHangMonitor> {
        let bhm_chan = BackgroundHangMonitorChan::new(self.sender.clone(), component_id);

        #[cfg(all(target_os = "windows", not(target_arch = "aarch64")))]
        let sampler = crate::sampler_windows::WindowsSampler::new();
        #[cfg(target_os = "macos")]
        let sampler = crate::sampler_mac::MacOsSampler::new();
        #[cfg(all(
            target_os = "linux",
            not(any(target_arch = "arm", target_arch = "aarch64"))
        ))]
        let sampler = crate::sampler_linux::LinuxSampler::new();
        #[cfg(any(target_os = "android", target_arch = "arm", target_arch = "aarch64"))]
        let sampler = crate::sampler::DummySampler::new();

        bhm_chan.send(MonitoredComponentMsg::Register(
            sampler,
            thread::current().name().map(str::to_owned),
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
    Register(Box<Sampler>, Option<String>, Duration, Duration),
    /// Unregister component for monitoring.
    Unregister,
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
    fn unregister(&self) {
        let msg = MonitoredComponentMsg::Unregister;
        self.send(msg);
    }
}

struct MonitoredComponent {
    sampler: Box<Sampler>,
    last_activity: Instant,
    last_annotation: Option<HangAnnotation>,
    transient_hang_timeout: Duration,
    permanent_hang_timeout: Duration,
    sent_transient_alert: bool,
    sent_permanent_alert: bool,
    is_waiting: bool,
}

struct Sample(MonitoredComponentId, Instant, NativeStack);

pub struct BackgroundHangMonitorWorker {
    component_names: HashMap<MonitoredComponentId, String>,
    monitored_components: HashMap<MonitoredComponentId, MonitoredComponent>,
    constellation_chan: IpcSender<HangMonitorAlert>,
    port: Receiver<(MonitoredComponentId, MonitoredComponentMsg)>,
    control_port: Receiver<SamplerControlMsg>,
    sampling_duration: Option<Duration>,
    sampling_max_duration: Option<Duration>,
    last_sample: Instant,
    creation: Instant,
    sampling_baseline: Instant,
    samples: VecDeque<Sample>,
}

impl BackgroundHangMonitorWorker {
    pub fn new(
        constellation_chan: IpcSender<HangMonitorAlert>,
        control_port: IpcReceiver<SamplerControlMsg>,
        port: Receiver<(MonitoredComponentId, MonitoredComponentMsg)>,
    ) -> Self {
        let control_port = ROUTER.route_ipc_receiver_to_new_crossbeam_receiver(control_port);
        Self {
            component_names: Default::default(),
            monitored_components: Default::default(),
            constellation_chan,
            port,
            control_port,
            sampling_duration: None,
            sampling_max_duration: None,
            last_sample: Instant::now(),
            sampling_baseline: Instant::now(),
            creation: Instant::now(),
            samples: Default::default(),
        }
    }

    fn finish_sampled_profile(&mut self) {
        let mut bytes = vec![];
        bytes.extend(
            format!(
                "{{ \"rate\": {}, \"start\": {}, \"data\": [\n",
                self.sampling_duration.unwrap().as_millis(),
                (self.sampling_baseline - self.creation).as_millis(),
            )
            .as_bytes(),
        );

        let mut first = true;
        let to_resolve = self.samples.len();
        for (i, Sample(id, instant, stack)) in self.samples.drain(..).enumerate() {
            println!("Resolving {}/{}", i + 1, to_resolve);
            let profile = stack.to_hangprofile();
            let name = match self.component_names.get(&id) {
                Some(ref s) => format!("\"{}\"", s),
                None => format!("null"),
            };
            let json = format!(
                "{}{{ \"name\": {}, \"namespace\": {}, \"index\": {}, \"type\": \"{:?}\", \
                 \"time\": {}, \"frames\": {} }}",
                if !first { ",\n" } else { "" },
                name,
                id.0.namespace_id.0,
                id.0.index.0.get(),
                id.1,
                (instant - self.sampling_baseline).as_millis(),
                serde_json::to_string(&profile.backtrace).unwrap(),
            );
            bytes.extend(json.as_bytes());
            first = false;
        }

        bytes.extend(b"\n] }");
        let _ = self
            .constellation_chan
            .send(HangMonitorAlert::Profile(bytes));
    }

    pub fn run(&mut self) -> bool {
        let timeout = if let Some(duration) = self.sampling_duration {
            duration
                .checked_sub(Instant::now() - self.last_sample)
                .unwrap_or_else(|| Duration::from_millis(0))
        } else {
            Duration::from_millis(100)
        };
        let received = select! {
            recv(self.port) -> event => {
                match event {
                    Ok(msg) => Some(msg),
                    // Our sender has been dropped, quit.
                    Err(_) => return false,
                }
            },
            recv(self.control_port) -> event => {
                match event {
                    Ok(SamplerControlMsg::Enable(rate, max_duration)) => {
                        println!("Enabling profiler.");
                        self.sampling_duration = Some(rate);
                        self.sampling_max_duration = Some(max_duration);
                        self.sampling_baseline = Instant::now();
                        None
                    }
                    Ok(SamplerControlMsg::Disable) => {
                        println!("Disabling profiler.");
                        self.finish_sampled_profile();
                        self.sampling_duration = None;
                        None
                    }
                    Err(_) => return false,
                }
            }
            recv(after(timeout)) -> _ => None,
        };
        if let Some(msg) = received {
            self.handle_msg(msg);
            while let Ok(another_msg) = self.port.try_recv() {
                // Handle any other incoming messages,
                // before performing a hang checkpoint.
                self.handle_msg(another_msg);
            }
        }

        if let Some(duration) = self.sampling_duration {
            let now = Instant::now();
            if now - self.last_sample > duration {
                self.sample();
                self.last_sample = now;
            }
        } else {
            self.perform_a_hang_monitor_checkpoint();
        }
        true
    }

    fn handle_msg(&mut self, msg: (MonitoredComponentId, MonitoredComponentMsg)) {
        match msg {
            (
                component_id,
                MonitoredComponentMsg::Register(
                    sampler,
                    name,
                    transient_hang_timeout,
                    permanent_hang_timeout,
                ),
            ) => {
                let component = MonitoredComponent {
                    sampler,
                    last_activity: Instant::now(),
                    last_annotation: None,
                    transient_hang_timeout,
                    permanent_hang_timeout,
                    sent_transient_alert: false,
                    sent_permanent_alert: false,
                    is_waiting: true,
                };
                if let Some(name) = name {
                    self.component_names.insert(component_id.clone(), name);
                }
                assert!(
                    self.monitored_components
                        .insert(component_id, component)
                        .is_none(),
                    "This component was already registered for monitoring."
                );
            },
            (component_id, MonitoredComponentMsg::Unregister) => {
                let _ = self
                    .monitored_components
                    .remove_entry(&component_id)
                    .expect("Received Unregister for an unknown component");
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
                let profile = match monitored.sampler.suspend_and_sample_thread() {
                    Ok(native_stack) => Some(native_stack.to_hangprofile()),
                    Err(()) => None,
                };
                let _ = self
                    .constellation_chan
                    .send(HangMonitorAlert::Hang(HangAlert::Permanent(
                        component_id.clone(),
                        last_annotation,
                        profile,
                    )));
                monitored.sent_permanent_alert = true;
                continue;
            }
            if monitored.last_activity.elapsed() > monitored.transient_hang_timeout {
                if monitored.sent_transient_alert {
                    continue;
                }
                let _ = self
                    .constellation_chan
                    .send(HangMonitorAlert::Hang(HangAlert::Transient(
                        component_id.clone(),
                        last_annotation,
                    )));
                monitored.sent_transient_alert = true;
            }
        }
    }

    fn sample(&mut self) {
        for (component_id, monitored) in self.monitored_components.iter_mut() {
            let instant = Instant::now();
            if let Ok(stack) = monitored.sampler.suspend_and_sample_thread() {
                if self.sampling_baseline.elapsed() >
                    self.sampling_max_duration
                        .expect("Max duration has been set")
                {
                    // Buffer is full, start discarding older samples.
                    self.samples.pop_front();
                }
                self.samples
                    .push_back(Sample(component_id.clone(), instant, stack));
            }
        }
    }
}
