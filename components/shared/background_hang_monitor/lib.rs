/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

#![deny(unsafe_code)]

//! An API interface to the BackgroundHangMonitor.

use std::time::Duration;
use std::{fmt, mem};

use base::id::PipelineId;
use ipc_channel::ipc::IpcSender;
use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, Debug, Deserialize, Serialize)]
/// The equivalent of script::script_runtime::ScriptEventCategory
pub enum ScriptHangAnnotation {
    AttachLayout,
    ConstellationMsg,
    DevtoolsMsg,
    DocumentEvent,
    DomEvent,
    FileRead,
    FormPlannedNavigation,
    ImageCacheMsg,
    InputEvent,
    HistoryEvent,
    NetworkEvent,
    Resize,
    ScriptEvent,
    SetScrollState,
    SetViewport,
    StylesheetLoad,
    TimerEvent,
    UpdateReplacedElement,
    WebSocketEvent,
    WorkerEvent,
    WorkletEvent,
    ServiceWorkerEvent,
    EnterFullscreen,
    ExitFullscreen,
    WebVREvent,
    PerformanceTimelineTask,
    PortMessage,
    WebGPUMsg,
}

#[derive(Clone, Copy, Debug, Deserialize, Serialize)]
pub enum HangAnnotation {
    Script(ScriptHangAnnotation),
}

/// Hang-alerts are sent by the monitor to the constellation.
#[derive(Deserialize, Serialize)]
pub enum HangMonitorAlert {
    /// A component hang has been detected.
    Hang(HangAlert),
    /// Report a completed sampled profile.
    Profile(Vec<u8>),
}

impl fmt::Debug for HangMonitorAlert {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            HangMonitorAlert::Hang(..) => write!(fmt, "Hang"),
            HangMonitorAlert::Profile(..) => write!(fmt, "Profile"),
        }
    }
}

/// Hang-alerts are sent by the monitor to the constellation.
#[derive(Deserialize, Serialize)]
pub enum HangAlert {
    /// Report a transient hang.
    Transient(MonitoredComponentId, HangAnnotation),
    /// Report a permanent hang.
    Permanent(MonitoredComponentId, HangAnnotation, Option<HangProfile>),
}

impl fmt::Debug for HangAlert {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
        let (annotation, profile) = match self {
            HangAlert::Transient(component_id, annotation) => {
                write!(
                    fmt,
                    "\n The following component is experiencing a transient hang: \n {:?}",
                    component_id
                )?;
                (*annotation, None)
            },
            HangAlert::Permanent(component_id, annotation, profile) => {
                write!(
                    fmt,
                    "\n The following component is experiencing a permanent hang: \n {:?}",
                    component_id
                )?;
                (*annotation, profile.clone())
            },
        };

        write!(fmt, "\n Annotation for the hang:\n{:?}", annotation)?;
        if let Some(profile) = profile {
            write!(fmt, "\n {:?}", profile)?;
        }

        Ok(())
    }
}

#[derive(Clone, Deserialize, Serialize)]
pub struct HangProfileSymbol {
    pub name: Option<String>,
    pub filename: Option<String>,
    pub lineno: Option<u32>,
}

#[derive(Clone, Deserialize, Serialize)]
/// Info related to the activity of an hanging component.
pub struct HangProfile {
    pub backtrace: Vec<HangProfileSymbol>,
}

impl fmt::Debug for HangProfile {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
        let hex_width = mem::size_of::<usize>() * 2 + 2;

        write!(fmt, "HangProfile backtrace:")?;

        if self.backtrace.is_empty() {
            write!(fmt, "backtrace failed to resolve")?;
            return Ok(());
        }

        for symbol in self.backtrace.iter() {
            write!(fmt, "\n      {:1$}", "", hex_width)?;

            if let Some(ref name) = symbol.name {
                write!(fmt, " - {}", name)?;
            } else {
                write!(fmt, " - <unknown>")?;
            }

            if let (Some(ref file), Some(ref line)) = (symbol.filename.as_ref(), symbol.lineno) {
                write!(fmt, "\n      {:3$}at {}:{}", "", file, line, hex_width)?;
            }
        }

        Ok(())
    }
}

#[derive(Clone, Debug, Deserialize, Eq, Hash, PartialEq, Serialize)]
pub enum MonitoredComponentType {
    Script,
}

#[derive(Clone, Debug, Deserialize, Eq, Hash, PartialEq, Serialize)]
pub struct MonitoredComponentId(pub PipelineId, pub MonitoredComponentType);

/// A handle to register components for hang monitoring,
/// and to receive a means to communicate with the underlying hang monitor worker.
pub trait BackgroundHangMonitorRegister: BackgroundHangMonitorClone + Send {
    /// Register a component for hang monitoring:
    /// to be called from within the thread to be monitored for hangs.
    fn register_component(
        &self,
        component: MonitoredComponentId,
        transient_hang_timeout: Duration,
        permanent_hang_timeout: Duration,
        exit_signal: Option<Box<dyn BackgroundHangMonitorExitSignal>>,
    ) -> Box<dyn BackgroundHangMonitor>;
}

impl Clone for Box<dyn BackgroundHangMonitorRegister> {
    fn clone(&self) -> Box<dyn BackgroundHangMonitorRegister> {
        self.clone_box()
    }
}

pub trait BackgroundHangMonitorClone {
    fn clone_box(&self) -> Box<dyn BackgroundHangMonitorRegister>;
}

/// Proxy methods to communicate with the background hang monitor
pub trait BackgroundHangMonitor {
    /// Notify the start of handling an event.
    fn notify_activity(&self, annotation: HangAnnotation);
    /// Notify the start of waiting for a new event to come in.
    fn notify_wait(&self);
    /// Unregister the component from monitor.
    fn unregister(&self);
}

/// A means for the BHM to signal a monitored component to exit.
/// Useful when the component is hanging, and cannot be notified via the usual way.
/// The component should implement this in a way allowing for the signal to be received when hanging,
/// if at all.
pub trait BackgroundHangMonitorExitSignal: Send {
    /// Called by the BHM, to notify the monitored component to exit.
    fn signal_to_exit(&self);
}

/// Messages to control the sampling profiler.
#[derive(Deserialize, Serialize)]
pub enum BackgroundHangMonitorControlMsg {
    /// Enable the sampler, with a given sampling rate and max total sampling duration.
    EnableSampler(Duration, Duration),
    DisableSampler,
    /// Exit, and propagate the signal to monitored components.
    Exit(IpcSender<()>),
}
