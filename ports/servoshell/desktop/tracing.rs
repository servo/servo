/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

/// Log an event from winit ([winit::event::Event]) at trace level.
/// - To disable tracing: RUST_LOG='servoshell<winit@=off'
/// - To enable tracing: RUST_LOG='servoshell<winit@'
/// - Recommended filters when tracing is enabled:
///   - servoshell<winit@DeviceEvent=off
///   - servoshell<winit@MainEventsCleared=off
///   - servoshell<winit@NewEvents(WaitCancelled)=off
///   - servoshell<winit@RedrawEventsCleared=off
///   - servoshell<winit@RedrawRequested=off
///   - servoshell<winit@UserEvent(WakerEvent)=off
///   - servoshell<winit@WindowEvent(AxisMotion)=off
///   - servoshell<winit@WindowEvent(CursorMoved)=off
macro_rules! trace_winit_event {
    // This macro only exists to put the docs in the same file as the target prefix,
    // so the macro definition is always the same.
    ($event:expr, $($rest:tt)+) => {
        ::log::trace!(target: $crate::desktop::tracing::LogTarget::log_target(&$event), $($rest)+)
    };
}

pub(crate) use trace_winit_event;

/// Get the log target for an event, as a static string.
pub(crate) trait LogTarget {
    fn log_target(&self) -> &'static str;
}

mod from_winit {
    use super::LogTarget;
    use crate::desktop::events_loop::AppEvent;

    macro_rules! target {
        ($($name:literal)+) => {
            concat!("servoshell<winit@", $($name),+)
        };
    }

    impl LogTarget for winit::event::Event<AppEvent> {
        fn log_target(&self) -> &'static str {
            use winit::event::StartCause;
            match self {
                Self::NewEvents(start_cause) => match start_cause {
                    StartCause::ResumeTimeReached { .. } => target!("NewEvents(ResumeTimeReached)"),
                    StartCause::WaitCancelled { .. } => target!("NewEvents(WaitCancelled)"),
                    StartCause::Poll => target!("NewEvents(Poll)"),
                    StartCause::Init => target!("NewEvents(Init)"),
                },
                Self::WindowEvent { event, .. } => event.log_target(),
                Self::DeviceEvent { .. } => target!("DeviceEvent"),
                Self::UserEvent(AppEvent::WakerEvent) => target!("UserEvent(WakerEvent)"),
                Self::UserEvent(AppEvent::Accessibility(..)) => target!("UserEvent(Accessibility)"),
                Self::Suspended => target!("Suspended"),
                Self::Resumed => target!("Resumed"),
                Self::AboutToWait => target!("AboutToWait"),
                Self::LoopExiting => target!("LoopExiting"),
                Self::MemoryWarning => target!("MemoryWarning"),
            }
        }
    }

    impl LogTarget for winit::event::WindowEvent {
        fn log_target(&self) -> &'static str {
            macro_rules! target_variant {
                ($name:literal) => {
                    target!("WindowEvent(" $name ")")
                };
            }
            match self {
                Self::ActivationTokenDone { .. } => target!("ActivationTokenDone"),
                Self::Resized(..) => target_variant!("Resized"),
                Self::Moved(..) => target_variant!("Moved"),
                Self::CloseRequested => target_variant!("CloseRequested"),
                Self::Destroyed => target_variant!("Destroyed"),
                Self::DroppedFile(..) => target_variant!("DroppedFile"),
                Self::HoveredFile(..) => target_variant!("HoveredFile"),
                Self::HoveredFileCancelled => target_variant!("HoveredFileCancelled"),
                Self::Focused(..) => target_variant!("Focused"),
                Self::KeyboardInput { .. } => target_variant!("KeyboardInput"),
                Self::ModifiersChanged(..) => target_variant!("ModifiersChanged"),
                Self::Ime(..) => target_variant!("Ime"),
                Self::CursorMoved { .. } => target_variant!("CursorMoved"),
                Self::CursorEntered { .. } => target_variant!("CursorEntered"),
                Self::CursorLeft { .. } => target_variant!("CursorLeft"),
                Self::MouseWheel { .. } => target_variant!("MouseWheel"),
                Self::MouseInput { .. } => target_variant!("MouseInput"),
                Self::PanGesture { .. } => target_variant!("PanGesture"),
                Self::PinchGesture { .. } => target_variant!("PinchGesture"),
                Self::DoubleTapGesture { .. } => target_variant!("DoubleTapGesture"),
                Self::RotationGesture { .. } => target_variant!("RotationGesture"),
                Self::TouchpadPressure { .. } => target_variant!("TouchpadPressure"),
                Self::AxisMotion { .. } => target_variant!("AxisMotion"),
                Self::Touch(..) => target_variant!("Touch"),
                Self::ScaleFactorChanged { .. } => target_variant!("ScaleFactorChanged"),
                Self::ThemeChanged(..) => target_variant!("ThemeChanged"),
                Self::Occluded(..) => target_variant!("Occluded"),
                Self::RedrawRequested => target!("RedrawRequested"),
            }
        }
    }
}
