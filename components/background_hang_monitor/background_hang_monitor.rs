/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use std::collections::{HashMap, VecDeque};
use std::thread::{self, Builder, JoinHandle};
use std::time::{Duration, Instant};

use background_hang_monitor_api::{
    BackgroundHangMonitor, BackgroundHangMonitorClone, BackgroundHangMonitorControlMsg,
    BackgroundHangMonitorExitSignal, BackgroundHangMonitorRegister, HangAlert, HangAnnotation,
    HangMonitorAlert, MonitoredComponentId,
};
use crossbeam_channel::{Receiver, Sender, after, never, select, unbounded};
use ipc_channel::ipc::{IpcReceiver, IpcSender};
use ipc_channel::router::ROUTER;

use crate::sampler::{NativeStack, Sampler};

#[derive(Clone)]
pub struct HangMonitorRegister {
    sender: MonitoredComponentSender,
    monitoring_enabled: bool,
}

impl HangMonitorRegister {
    /// Start a new hang monitor worker, and return a handle to register components for monitoring,
    /// as well as a join handle on the worker thread.
    pub fn init(
        constellation_chan: IpcSender<HangMonitorAlert>,
        control_port: IpcReceiver<BackgroundHangMonitorControlMsg>,
        monitoring_enabled: bool,
    ) -> (Box<dyn BackgroundHangMonitorRegister>, JoinHandle<()>) {
        let (sender, port) = unbounded();
        let sender_clone = sender.clone();

        let join_handle = Builder::new()
            .name("BackgroundHangMonitor".to_owned())
            .spawn(move || {
                let mut monitor = BackgroundHangMonitorWorker::new(
                    constellation_chan,
                    control_port,
                    port,
                    monitoring_enabled,
                );
                while monitor.run() {
                    // Monitoring until all senders have been dropped...
                }
            })
            .expect("Couldn't start BHM worker.");
        (
            Box::new(HangMonitorRegister {
                sender: sender_clone,
                monitoring_enabled,
            }),
            join_handle,
        )
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
        exit_signal: Box<dyn BackgroundHangMonitorExitSignal>,
    ) -> Box<dyn BackgroundHangMonitor> {
        let bhm_chan = BackgroundHangMonitorChan::new(
            self.sender.clone(),
            component_id,
            self.monitoring_enabled,
        );

        #[cfg(all(
            feature = "sampler",
            target_os = "windows",
            any(target_arch = "x86_64", target_arch = "x86")
        ))]
        let sampler = crate::sampler_windows::WindowsSampler::new_boxed();
        #[cfg(all(feature = "sampler", target_os = "macos"))]
        let sampler = crate::sampler_mac::MacOsSampler::new_boxed();
        #[cfg(all(feature = "sampler", target_os = "android"))]
        let sampler = crate::sampler_linux::LinuxSampler::new_boxed();
        #[cfg(all(
            feature = "sampler",
            target_os = "linux",
            not(any(
                target_arch = "arm",
                target_arch = "aarch64",
                target_env = "ohos",
                target_env = "musl"
            )),
        ))]
        let sampler = crate::sampler_linux::LinuxSampler::new_boxed();
        #[cfg(any(
            not(feature = "sampler"),
            all(
                target_os = "linux",
                any(
                    target_arch = "arm",
                    target_arch = "aarch64",
                    target_env = "ohos",
                    target_env = "musl"
                )
            ),
        ))]
        let sampler = crate::sampler::DummySampler::new_boxed();

        bhm_chan.send(MonitoredComponentMsg::Register(
            sampler,
            thread::current().name().map(str::to_owned),
            transient_hang_timeout,
            permanent_hang_timeout,
            exit_signal,
        ));
        Box::new(bhm_chan)
    }
}

impl BackgroundHangMonitorClone for HangMonitorRegister {
    fn clone_box(&self) -> Box<dyn BackgroundHangMonitorRegister> {
        Box::new(self.clone())
    }
}

/// Messages sent from monitored components to the monitor.
enum MonitoredComponentMsg {
    /// Register component for monitoring,
    Register(
        Box<dyn Sampler>,
        Option<String>,
        Duration,
        Duration,
        Box<dyn BackgroundHangMonitorExitSignal>,
    ),
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
struct BackgroundHangMonitorChan {
    sender: MonitoredComponentSender,
    component_id: MonitoredComponentId,
    monitoring_enabled: bool,
}

impl BackgroundHangMonitorChan {
    fn new(
        sender: MonitoredComponentSender,
        component_id: MonitoredComponentId,
        monitoring_enabled: bool,
    ) -> Self {
        BackgroundHangMonitorChan {
            sender,
            component_id,
            monitoring_enabled,
        }
    }

    fn send(&self, msg: MonitoredComponentMsg) {
        self.sender
            .send((self.component_id.clone(), msg))
            .expect("BHM is gone");
    }
}

impl BackgroundHangMonitor for BackgroundHangMonitorChan {
    fn notify_activity(&self, annotation: HangAnnotation) {
        if self.monitoring_enabled {
            let msg = MonitoredComponentMsg::NotifyActivity(annotation);
            self.send(msg);
        }
    }
    fn notify_wait(&self) {
        if self.monitoring_enabled {
            let msg = MonitoredComponentMsg::NotifyWait;
            self.send(msg);
        }
    }
    fn unregister(&self) {
        let msg = MonitoredComponentMsg::Unregister;
        self.send(msg);
    }
}

struct MonitoredComponent {
    sampler: Box<dyn Sampler>,
    last_activity: Instant,
    last_annotation: Option<HangAnnotation>,
    transient_hang_timeout: Duration,
    permanent_hang_timeout: Duration,
    sent_transient_alert: bool,
    sent_permanent_alert: bool,
    is_waiting: bool,
    exit_signal: Box<dyn BackgroundHangMonitorExitSignal>,
}

struct Sample(MonitoredComponentId, Instant, NativeStack);

struct BackgroundHangMonitorWorker {
    component_names: HashMap<MonitoredComponentId, String>,
    monitored_components: HashMap<MonitoredComponentId, MonitoredComponent>,
    constellation_chan: IpcSender<HangMonitorAlert>,
    port: Receiver<(MonitoredComponentId, MonitoredComponentMsg)>,
    control_port: Receiver<BackgroundHangMonitorControlMsg>,
    sampling_duration: Option<Duration>,
    sampling_max_duration: Option<Duration>,
    last_sample: Instant,
    creation: Instant,
    sampling_baseline: Instant,
    samples: VecDeque<Sample>,
    monitoring_enabled: bool,
    shutting_down: bool,
}

type MonitoredComponentSender = Sender<(MonitoredComponentId, MonitoredComponentMsg)>;
type MonitoredComponentReceiver = Receiver<(MonitoredComponentId, MonitoredComponentMsg)>;

impl BackgroundHangMonitorWorker {
    fn new(
        constellation_chan: IpcSender<HangMonitorAlert>,
        control_port: IpcReceiver<BackgroundHangMonitorControlMsg>,
        port: MonitoredComponentReceiver,
        monitoring_enabled: bool,
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
            monitoring_enabled,
            shutting_down: Default::default(),
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
                None => "null".to_string(),
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

    fn run(&mut self) -> bool {
        let tick = if let Some(duration) = self.sampling_duration {
            let duration = duration
                .checked_sub(Instant::now() - self.last_sample)
                .unwrap_or_else(|| Duration::from_millis(0));
            after(duration)
        } else if self.monitoring_enabled {
            after(Duration::from_millis(100))
        } else {
            never()
        };

        let received = select! {
            recv(self.port) -> event => {
                if let Ok(event) = event {
                    Some(event)
                } else {
                    // All senders have dropped,
                    // which means all monitored components have shut down,
                    // and so we can as well.
                    return false;
                }
            },
            recv(self.control_port) -> event => {
                match event {
                    Ok(BackgroundHangMonitorControlMsg::ToggleSampler(rate, max_duration)) => {
                        if self.sampling_duration.is_some() {
                            println!("Enabling profiler.");
                            self.finish_sampled_profile();
                            self.sampling_duration = None;
                        } else {
                            println!("Disabling profiler.");
                            self.sampling_duration = Some(rate);
                            self.sampling_max_duration = Some(max_duration);
                            self.sampling_baseline = Instant::now();
                        }
                        None
                    },
                    Ok(BackgroundHangMonitorControlMsg::Exit) => {
                        for component in self.monitored_components.values_mut() {
                            component.exit_signal.signal_to_exit();
                        }

                        // Note the start of shutdown,
                        // to ensure exit propagates,
                        // even to components that have yet to register themselves,
                        // from this point on.
                        self.shutting_down = true;

                        // Keep running; this worker thread will shutdown
                        // when the monitored components have shutdown,
                        // which we know has happened when `self.port` disconnects.
                        None
                    },
                    Err(_) => return false,
                }
            }
            recv(tick) -> _ => None,
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
                    exit_signal,
                ),
            ) => {
                // If we are shutting down,
                // propagate it to the component,
                // and register it(the component will unregister itself
                // as part of handling the exit).
                if self.shutting_down {
                    exit_signal.signal_to_exit();
                }

                let component = MonitoredComponent {
                    sampler,
                    last_activity: Instant::now(),
                    last_annotation: None,
                    transient_hang_timeout,
                    permanent_hang_timeout,
                    sent_transient_alert: false,
                    sent_permanent_alert: false,
                    is_waiting: true,
                    exit_signal,
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
                self.monitored_components
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
