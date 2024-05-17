/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use std::cell::Cell;
use std::collections::{HashMap, VecDeque};
use std::sync::{Arc, Weak};
use std::thread;
use std::time::{Duration, Instant};

use background_hang_monitor_api::{
    BackgroundHangMonitor, BackgroundHangMonitorClone, BackgroundHangMonitorControlMsg,
    BackgroundHangMonitorExitSignal, BackgroundHangMonitorRegister, HangAlert, HangAnnotation,
    HangMonitorAlert, MonitoredComponentId,
};
use crossbeam_channel::{after, never, select, unbounded, Receiver, Sender};
use ipc_channel::ipc::{IpcReceiver, IpcSender};
use ipc_channel::router::ROUTER;
use log::warn;

use crate::sampler::{NativeStack, Sampler};

#[derive(Clone)]
pub struct HangMonitorRegister {
    sender: Weak<Sender<(MonitoredComponentId, MonitoredComponentMsg)>>,
    tether: Sender<Never>,
    monitoring_enabled: bool,
}

impl HangMonitorRegister {
    /// Start a new hang monitor worker, and return a handle to register components for monitoring.
    pub fn init(
        constellation_chan: IpcSender<HangMonitorAlert>,
        control_port: IpcReceiver<BackgroundHangMonitorControlMsg>,
        monitoring_enabled: bool,
    ) -> Box<dyn BackgroundHangMonitorRegister> {
        // Create a channel to pass messages of type `MonitoredComponentMsg`.
        // See the discussion in `<HangMonitorRegister as
        // BackgroundHangMonitorRegister>::register_component` for why we wrap
        // the sender with `Arc` and why `HangMonitorRegister` only maintains
        // a weak reference to it.
        let (sender, port) = unbounded();
        let sender = Arc::new(sender);
        let sender_weak = Arc::downgrade(&sender);

        // Create a "tether" channel, whose sole purpose is to keep the worker
        // thread alive. The worker thread will terminates when all copies of
        // `tether` are dropped.
        let (tether, tether_port) = unbounded();

        let _ = thread::Builder::new()
            .name("BackgroundHangMonitor".to_owned())
            .spawn(move || {
                let mut monitor = BackgroundHangMonitorWorker::new(
                    constellation_chan,
                    control_port,
                    (sender, port),
                    tether_port,
                    monitoring_enabled,
                );
                while monitor.run() {
                    // Monitoring until all senders have been dropped...
                }
            })
            .expect("Couldn't start BHM worker.");
        Box::new(HangMonitorRegister {
            sender: sender_weak,
            tether,
            monitoring_enabled,
        })
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
        exit_signal: Option<Box<dyn BackgroundHangMonitorExitSignal>>,
    ) -> Box<dyn BackgroundHangMonitor> {
        let bhm_chan = BackgroundHangMonitorChan::new(
            self.sender.clone(),
            self.tether.clone(),
            component_id,
            self.monitoring_enabled,
        );

        #[cfg(all(
            target_os = "windows",
            any(target_arch = "x86_64", target_arch = "x86")
        ))]
        let sampler = crate::sampler_windows::WindowsSampler::new_boxed();
        #[cfg(target_os = "macos")]
        let sampler = crate::sampler_mac::MacOsSampler::new_boxed();
        #[cfg(all(
            target_os = "linux",
            not(any(target_arch = "arm", target_arch = "aarch64"))
        ))]
        let sampler = crate::sampler_linux::LinuxSampler::new_boxed();
        #[cfg(any(
            target_os = "android",
            all(target_os = "linux", any(target_arch = "arm", target_arch = "aarch64"))
        ))]
        let sampler = crate::sampler::DummySampler::new_boxed();

        // When a component is registered, and there's an exit request that
        // reached BHM, we want an exit signal to be delivered to the
        // component's exit signal handler eventually. However, there's a race
        // condition between the reception of `BackgroundHangMonitorControlMsg::
        // Exit` and `MonitoredComponentMsg::Register` that needs to handled
        // carefully. When the worker receives an `Exit` message, it stops
        // processing messages, and any further `Register` messages sent to the
        // worker thread are ignored. If the submissions of `Exit` and
        // `Register` messages are far apart enough, the channel is closed by
        // the time the client attempts to send a `Register` message, and
        // therefore the client can figure out by `Sender::send`'s return value
        // that it must deliver an exit signal. However, if these message
        // submissions are close enough, the `Register` message is still sent,
        // but the worker thread might exit before it sees the message, leaving
        // the message unprocessed and the exit signal unsent.
        //
        // To fix this, we wrap the exit signal handler in an RAII wrapper of
        // type `SignalToExitOnDrop` to automatically send a signal when it's
        // dropped. This way, we can make sure the exit signal is sent even if
        // the message couldn't reach the worker thread and be processed.
        //
        // However, as it turns out, `crossbeam-channel`'s channels don't drop
        // remaining messages until all associated senders *and* receivers are
        // dropped. This means the exit signal won't be delivered as long as
        // there's at least one `HangMonitorRegister` or
        // `BackgroundHangMonitorChan` maintaining a copy of the sender. To work
        // around this and guarantee a rapid delivery of the exit signal, the
        // sender is wrapped in `Arc`, and only the worker thread maintains a
        // strong reference, thus ensuring both the sender and receiver are
        // dropped as soon as the worker thread exits.
        let exit_signal = SignalToExitOnDrop(exit_signal);

        // If the tether is dropped after this call, the worker thread might
        // exit before processing the `Register` message because there's no
        // implicit ordering guarantee between two channels. If this happens,
        // an exit signal will be sent despite we haven't received a
        // corresponding exit request. To enforce the correct ordering and
        // prevent a false exit signal from being sent, we include a copy of
        // `self.tether` in the `Register` message.
        let tether = self.tether.clone();

        bhm_chan.send(MonitoredComponentMsg::Register(
            sampler,
            thread::current().name().map(str::to_owned),
            transient_hang_timeout,
            permanent_hang_timeout,
            exit_signal,
            tether,
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
        SignalToExitOnDrop,
        Sender<Never>,
    ),
    /// Unregister component for monitoring.
    Unregister,
    /// Notify start of new activity for a given component,
    NotifyActivity(HangAnnotation),
    /// Notify start of waiting for a new task to come-in for processing.
    NotifyWait,
}

/// Stable equivalent to the `!` type
enum Never {}

/// A wrapper around a sender to the monitor,
/// which will send the Id of the monitored component along with each message,
/// and keep track of whether the monitor is still listening on the other end.
struct BackgroundHangMonitorChan {
    sender: Weak<Sender<(MonitoredComponentId, MonitoredComponentMsg)>>,
    _tether: Sender<Never>,
    component_id: MonitoredComponentId,
    disconnected: Cell<bool>,
    monitoring_enabled: bool,
}

impl BackgroundHangMonitorChan {
    fn new(
        sender: Weak<Sender<(MonitoredComponentId, MonitoredComponentMsg)>>,
        tether: Sender<Never>,
        component_id: MonitoredComponentId,
        monitoring_enabled: bool,
    ) -> Self {
        BackgroundHangMonitorChan {
            sender,
            _tether: tether,
            component_id,
            disconnected: Default::default(),
            monitoring_enabled,
        }
    }

    fn send(&self, msg: MonitoredComponentMsg) {
        if self.disconnected.get() {
            return;
        }

        // The worker thread owns both the receiver *and* the only strong
        // reference to the sender. An `upgrade` failure means the latter is
        // gone, and a `send` failure means the former is gone. They are dropped
        // simultaneously, but we might observe an intermediate state.
        if self
            .sender
            .upgrade()
            .and_then(|sender| sender.send((self.component_id.clone(), msg)).ok())
            .is_none()
        {
            warn!("BackgroundHangMonitor has gone away");
            self.disconnected.set(true);
        }
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

/// Wraps [`BackgroundHangMonitorExitSignal`] and calls `signal_to_exit` when
/// dropped.
struct SignalToExitOnDrop(Option<Box<dyn BackgroundHangMonitorExitSignal>>);

impl SignalToExitOnDrop {
    /// Call `BackgroundHangMonitorExitSignal::signal_to_exit` now.
    fn signal_to_exit(&mut self) {
        if let Some(signal) = self.0.take() {
            signal.signal_to_exit();
        }
    }

    /// Disassociate `BackgroundHangMonitorExitSignal` from itself, preventing
    /// `BackgroundHangMonitorExitSignal::signal_to_exit` from being called in
    /// the future.
    fn release(&mut self) {
        self.0 = None;
    }
}

impl Drop for SignalToExitOnDrop {
    #[inline]
    fn drop(&mut self) {
        self.signal_to_exit();
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
    exit_signal: SignalToExitOnDrop,
}

struct Sample(MonitoredComponentId, Instant, NativeStack);

struct BackgroundHangMonitorWorker {
    component_names: HashMap<MonitoredComponentId, String>,
    monitored_components: HashMap<MonitoredComponentId, MonitoredComponent>,
    constellation_chan: IpcSender<HangMonitorAlert>,
    port: Receiver<(MonitoredComponentId, MonitoredComponentMsg)>,
    _port_sender: Arc<Sender<(MonitoredComponentId, MonitoredComponentMsg)>>,
    tether_port: Receiver<Never>,
    control_port: Receiver<BackgroundHangMonitorControlMsg>,
    sampling_duration: Option<Duration>,
    sampling_max_duration: Option<Duration>,
    last_sample: Instant,
    creation: Instant,
    sampling_baseline: Instant,
    samples: VecDeque<Sample>,
    monitoring_enabled: bool,
}

type MonitoredComponentSender = Sender<(MonitoredComponentId, MonitoredComponentMsg)>;
type MonitoredComponentReceiver = Receiver<(MonitoredComponentId, MonitoredComponentMsg)>;

impl BackgroundHangMonitorWorker {
    fn new(
        constellation_chan: IpcSender<HangMonitorAlert>,
        control_port: IpcReceiver<BackgroundHangMonitorControlMsg>,
        (port_sender, port): (Arc<MonitoredComponentSender>, MonitoredComponentReceiver),
        tether_port: Receiver<Never>,
        monitoring_enabled: bool,
    ) -> Self {
        let control_port = ROUTER.route_ipc_receiver_to_new_crossbeam_receiver(control_port);
        Self {
            component_names: Default::default(),
            monitored_components: Default::default(),
            constellation_chan,
            port,
            _port_sender: port_sender,
            tether_port,
            control_port,
            sampling_duration: None,
            sampling_max_duration: None,
            last_sample: Instant::now(),
            sampling_baseline: Instant::now(),
            creation: Instant::now(),
            samples: Default::default(),
            monitoring_enabled,
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
                // Since we own the `Arc<Sender<_>>`, the channel never
                // gets disconnected.
                Some(event.unwrap())
            },
            recv(self.tether_port) -> _ => {
                // This arm can only reached by a tether disconnection
                // All associated `HangMonitorRegister` and
                // `BackgroundHangMonitorChan` have been dropped. Suppress
                // `signal_to_exit` and exit the BHM.
                for component in self.monitored_components.values_mut() {
                    component.exit_signal.release();
                }
                return false;
            },
            recv(self.control_port) -> event => {
                match event {
                    Ok(BackgroundHangMonitorControlMsg::EnableSampler(rate, max_duration)) => {
                        println!("Enabling profiler.");
                        self.sampling_duration = Some(rate);
                        self.sampling_max_duration = Some(max_duration);
                        self.sampling_baseline = Instant::now();
                        None
                    },
                    Ok(BackgroundHangMonitorControlMsg::DisableSampler) => {
                        println!("Disabling profiler.");
                        self.finish_sampled_profile();
                        self.sampling_duration = None;
                        return true;
                    },
                    Ok(BackgroundHangMonitorControlMsg::Exit(sender)) => {
                        for component in self.monitored_components.values_mut() {
                            component.exit_signal.signal_to_exit();
                        }

                        // Confirm exit with to the constellation.
                        let _ = sender.send(());

                        // Also exit the BHM.
                        return false;
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
                    _tether,
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
                let (_, mut component) = self
                    .monitored_components
                    .remove_entry(&component_id)
                    .expect("Received Unregister for an unknown component");

                // Prevent `signal_to_exit` from being called
                component.exit_signal.release();
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
