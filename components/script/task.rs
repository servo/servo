/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

//! Machinery for [tasks](https://html.spec.whatwg.org/multipage/#concept-task).

use std::fmt;
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use msg::constellation_msg::PipelineId;

macro_rules! task {
    ($name:ident: move || $body:tt, $pipeline:stmt) => {{
        #[allow(non_camel_case_types)]
        struct $name<F>(F);
        impl<F> ::task::TaskOnce for $name<F>
        where
            F: ::std::ops::FnOnce() + Send,
        {
            fn name(&self) -> &'static str {
                stringify!($name)
            }

            fn run_once(self) {
                (self.0)();
            }

            fn task(&self) -> Option<PipelineId> {
                $pipeline
            }
        }
        $name(move || $body)
    }};
}

/// A task that can be run. The name method is for profiling purposes.
pub trait TaskOnce: Send {
    #[allow(unsafe_code)]
    fn name(&self) -> &'static str {
        #[cfg(feature = "unstable")]
        unsafe { ::std::intrinsics::type_name::<Self>() }
        #[cfg(not(feature = "unstable"))]
        { "(task name unknown)" }
    }

    fn run_once(self);

    fn pipeline(&self) -> Option<PipelineId> { None }
}

/// A boxed version of `TaskOnce`.
pub trait TaskBox: Send {
    fn name(&self) -> &'static str;

    fn run_box(self: Box<Self>);
}

impl<T> TaskBox for T
where
    T: TaskOnce,
{
    fn name(&self) -> &'static str {
        TaskOnce::name(self)
    }

    fn run_box(self: Box<Self>) {
        self.run_once()
    }
}

impl fmt::Debug for TaskBox {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
        fmt.debug_tuple(self.name()).field(&format_args!("...")).finish()
    }
}

/// Encapsulated state required to create cancellable tasks from non-script threads.
pub struct TaskCanceller {
    pub cancelled: Option<Arc<AtomicBool>>,
}

impl TaskCanceller {
    /// Returns a wrapped `task` that will be cancelled if the `TaskCanceller`
    /// says so.
    pub fn wrap_task<T>(&self, task: T) -> impl TaskOnce
    where
        T: TaskOnce,
    {
        CancellableTask {
            cancelled: self.cancelled.clone(),
            inner: task,
        }
    }
}

/// A task that can be cancelled by toggling a shared flag.
pub struct CancellableTask<T: TaskOnce> {
    cancelled: Option<Arc<AtomicBool>>,
    inner: T,
}

impl<T> CancellableTask<T>
where
    T: TaskOnce,
{
    fn is_cancelled(&self) -> bool {
        self.cancelled.as_ref().map_or(false, |cancelled| {
            cancelled.load(Ordering::SeqCst)
        })
    }
}

impl<T> TaskOnce for CancellableTask<T>
where
    T: TaskOnce,
{
    fn name(&self) -> &'static str {
        self.inner.name()
    }

    fn run_once(self) {
        if !self.is_cancelled() {
            self.inner.run_once()
        }
    }

    pub fn pipeline(&self) -> Option<PipelineId> {
        None
    }
}

// impl TaskOnce for ConstellationControlMsg {
//     fn pipeline(&self) -> Option<PipelineId> {
//         use script_thread::ConstellationControlMsg::*;
//         match *self {
//             NavigationResponse(id, _) => Some(id),
//             AttachLayout(ref new_layout_info) => Some(new_layout_info.new_pipeline_id),
//             Resize(id, ..) => Some(id),
//             ResizeInactive(id, ..) => Some(id),
//             ExitPipeline(id, ..) => Some(id),
//             ExitScriptThread => None,
//             SendEvent(id, ..) => Some(id),
//             Viewport(id, ..) => Some(id),
//             SetScrollState(id, ..) => Some(id),
//             GetTitle(id) => Some(id),
//             SetDocumentActivity(id, ..) => Some(id),
//             ChangeFrameVisibilityStatus(id, ..) => Some(id),
//             NotifyVisibilityChange(id, ..) => Some(id),
//             Navigate(id, ..) => Some(id),
//             PostMessage(id, ..) => Some(id),
//             MozBrowserEvent(id, ..) => Some(id),
//             UpdatePipelineId(_, _, id, _) => Some(id),
//             FocusIFrame(id, ..) => Some(id),
//             WebDriverScriptCommand(id, ..) => Some(id),
//             TickAllAnimations(id) => Some(id),
//             TransitionEnd(..) => None,
//             WebFontLoaded(id) => Some(id),
//             DispatchIFrameLoadEvent { .. } => None,
//             DispatchStorageEvent(id, ..) => Some(id),
//             ReportCSSError(id, ..) => Some(id),
//             Reload(id, ..) => Some(id),
//             WebVREvents(id, ..) => Some(id),
//             PaintMetric(..) => None,
//             InteractiveMetric(..) => None,
//         }
//     }
// }

// impl TaskOnce for MainThreadScriptMsg {
//     fn pipeline(&self) -> Option<PipelineId> {
//         use self::MainThreadScriptMsg::*;
//         match *self {
//             Common(_) => None,
//             ExitWindow(pipeline_id) => Some(pipeline_id),
//             Navigate(pipeline_id, ..) => Some(pipeline_id),
//             WorkletLoaded(pipeline_id) => Some(pipeline_id),
//             RegisterPaintWorklet { pipeline_id, .. } => Some(pipeline_id),
//             DispatchJobQueue { .. } => None,
//         }
//     }
// }

// impl TaskOnce for DevtoolScriptControlMsg {
//     fn pipeline(&self) -> Option<PipelineId> {
//         None
//     }
// }
