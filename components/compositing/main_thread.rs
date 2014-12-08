/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

//! The OS main thread.
//!
//! As a matter of policy, we run as little here as we can get away with.

use compositor_task::{CompositorProxy, Exit, PinchZoom, Refresh, Resize, Scroll, SendMouseEvent};
use compositor_task::{SendMouseMoveEvent, SynchronousRefresh, Zoom};
use geom::point::TypedPoint2D;
use geom::size::TypedSize2D;
use windowing::{mod, KeyEvent, IdleWindowEvent, LoadUrlWindowEvent, MouseWindowEvent};
use windowing::{MouseWindowEventClass, MouseWindowMoveEventClass, NavigationWindowEvent};
use windowing::{PinchZoomWindowEvent, QuitWindowEvent, RefreshWindowEvent, ResizeWindowEvent};
use windowing::{ScrollWindowEvent, SetPaintStateWindowEvent, SetReadyStateWindowEvent};
use windowing::{SynchronousRepaintWindowEvent, WindowEvent, WindowMethods, WindowNavigateMsg};
use windowing::{ZoomWindowEvent};

use servo_msg::compositor_msg::{PaintState, ReadyState, ScriptToMainThreadProxy};
use servo_msg::constellation_msg::{mod, ConstellationChan, ExitMsg, Key, KeyState, KeyModifiers};
use servo_msg::constellation_msg::{LoadData, LoadUrlMsg, NavigateMsg};
use layers::geometry::DevicePixel;
use std::comm;
use std::rc::Rc;
use url::Url;

/// Data that the main thread stores.
pub struct MainThread<W> {
    /// The application window.
    window: Option<Rc<W>>,

    /// A copy of the channel on which we can send messages to ourselves.
    main_thread_sender: Sender<WindowEvent>,

    /// The port on which we receive messages.
    main_thread_receiver: Receiver<WindowEvent>,

    /// The channel on which messages can be sent to the compositor. If `None`, we're running
    /// headless.
    compositor_proxy: Option<CompositorProxy>,

    /// The channel on which messages can be sent to the constellation.
    constellation_proxy: ConstellationChan,
}

impl<W> MainThread<W> where W: WindowMethods {
    pub fn new(window: Option<Rc<W>>,
               main_thread_sender: Sender<WindowEvent>,
               main_thread_receiver: Receiver<WindowEvent>,
               compositor_proxy: Option<CompositorProxy>,
               constellation_proxy: ConstellationChan)
               -> MainThread<W> {
        MainThread {
            window: window,
            main_thread_sender: main_thread_sender,
            main_thread_receiver: main_thread_receiver,
            compositor_proxy: compositor_proxy,
            constellation_proxy: constellation_proxy,
        }
    }

    /// Processes all events in the queue. Returns true if the browser is to continue processing
    /// events or false if the browser is to shut down.
    pub fn process_events(&mut self) -> bool {
        loop {
            let event = match self.main_thread_receiver.try_recv() {
                Err(_) => return true,
                Ok(event) => event,
            };
            match event {
                IdleWindowEvent => {}
                RefreshWindowEvent => self.on_refresh_window_event(),
                ResizeWindowEvent(size) => self.on_resize_window_event(size),
                LoadUrlWindowEvent(url_string) =>  self.on_load_url_window_event(url_string),
                MouseWindowEventClass(mouse_event) => {
                    self.on_mouse_window_event_class(mouse_event)
                }
                MouseWindowMoveEventClass(cursor) => self.on_mouse_window_move_event_class(cursor),
                ScrollWindowEvent(delta, cursor) => self.on_scroll_window_event(delta, cursor),
                ZoomWindowEvent(magnification) => self.on_zoom_window_event(magnification),
                PinchZoomWindowEvent(magnification) => {
                    self.on_pinch_zoom_window_event(magnification)
                }
                NavigationWindowEvent(direction) => self.on_navigation_window_event(direction),
                KeyEvent(key, state, modifiers) => self.on_key_event(key, state, modifiers),
                SetReadyStateWindowEvent(ready_state) => {
                    self.on_set_ready_state_event(ready_state)
                }
                SetPaintStateWindowEvent(state) => self.on_set_paint_state_event(state),
                SynchronousRepaintWindowEvent => self.on_synchronous_repaint_event(),
                QuitWindowEvent => {
                    self.on_quit_event();
                    return false
                }
            }
        }
    }

    /// Enqueues an event to be processed. The message will not be actually processed until
    /// `process_events()` is called.
    pub fn enqueue(&mut self, event: WindowEvent) {
        self.main_thread_sender.send(event)
    }

    // Event handlers follow:

    fn on_refresh_window_event(&mut self) {
        self.refresh()
    }

    fn on_resize_window_event(&mut self, new_size: TypedSize2D<DevicePixel, uint>) {
        match self.compositor_proxy {
            None => {}
            Some(ref mut compositor_proxy) => {
                compositor_proxy.send(Resize(new_size,
                                             self.window.as_ref().unwrap().hidpi_factor()))
            }
        }
    }

    fn on_load_url_window_event(&mut self, url_string: String) {
        debug!("osmain: loading URL `{:s}`", url_string);
        let msg = LoadUrlMsg(None, LoadData::new(Url::parse(url_string.as_slice()).unwrap()));
        let ConstellationChan(ref chan) = self.constellation_proxy;
        chan.send(msg);
    }

    fn on_mouse_window_event_class(&mut self, mouse_event: MouseWindowEvent) {
        match self.compositor_proxy {
            None => {}
            Some(ref mut compositor_proxy) => compositor_proxy.send(SendMouseEvent(mouse_event)),
        }
    }

    fn on_mouse_window_move_event_class(&mut self, cursor: TypedPoint2D<DevicePixel,f32>) {
        match self.compositor_proxy {
            None => {}
            Some(ref mut compositor_proxy) => compositor_proxy.send(SendMouseMoveEvent(cursor)),
        }
    }

    fn on_scroll_window_event(&mut self,
                              delta: TypedPoint2D<DevicePixel,f32>,
                              cursor: TypedPoint2D<DevicePixel,i32>) {
        match self.compositor_proxy {
            None => {}
            Some(ref mut compositor_proxy) => compositor_proxy.send(Scroll(delta, cursor)),
        }
    }

    fn on_zoom_window_event(&mut self, magnification: f32) {
        match self.compositor_proxy {
            None => {}
            Some(ref mut compositor_proxy) => compositor_proxy.send(Zoom(magnification)),
        }
    }

    // TODO(pcwalton): I think this should go through the same queuing as scroll events do.
    fn on_pinch_zoom_window_event(&mut self, magnification: f32) {
        match self.compositor_proxy {
            None => {}
            Some(ref mut compositor_proxy) => compositor_proxy.send(PinchZoom(magnification)),
        }
    }

    fn on_navigation_window_event(&self, direction: WindowNavigateMsg) {
        let direction = match direction {
            windowing::Forward => constellation_msg::Forward,
            windowing::Back => constellation_msg::Back,
        };
        let ConstellationChan(ref chan) = self.constellation_proxy;
        chan.send(NavigateMsg(direction))
    }

    fn on_key_event(&self, key: Key, state: KeyState, modifiers: KeyModifiers) {
        let ConstellationChan(ref chan) = self.constellation_proxy;
        chan.send(constellation_msg::KeyEvent(key, state, modifiers))
    }

    fn on_set_ready_state_event(&self, ready_state: ReadyState) {
        self.window.as_ref().unwrap().set_ready_state(ready_state)
    }

    fn on_set_paint_state_event(&self, paint_state: PaintState) {
        self.window.as_ref().unwrap().set_paint_state(paint_state)
    }

    fn on_synchronous_repaint_event(&mut self) {
        match self.compositor_proxy {
            None => {}
            Some(ref mut compositor_proxy) => {
                let (sender, receiver) = comm::channel();
                compositor_proxy.send(SynchronousRefresh(sender));
                receiver.recv()
            }
        }
    }

    fn on_quit_event(&mut self) {
        debug!("shutting down the constellation and compositor for QuitWindowEvent");

        // If the constellation itself initiated this quit message, it will have already shut
        // down. Use `send_opt` to avoid a panic in this case.
        let ConstellationChan(ref chan) = self.constellation_proxy;
        drop(chan.send_opt(ExitMsg));
        let (sender, receiver) = comm::channel();

        match self.compositor_proxy {
            None => {}
            Some(ref mut compositor_proxy) => {
                compositor_proxy.send(Exit(sender));
                receiver.recv();
            }
        }
    }

    // Helper methods follow:

    fn refresh(&mut self) {
        match self.compositor_proxy {
            None => {}
            Some(ref mut compositor_proxy) => compositor_proxy.send(Refresh),
        }
    }
}

/// Sends messages to the main thread. This is a trait supplied by the port because the method used
/// to communicate with the main thread may have to kick OS event loops awake, communicate cross-
/// process, and so forth.
pub trait MainThreadProxy {
    /// Sends an event to the main thread and kicks it awake. You should not use this method if you
    /// are on the main thread handling an event; instead, send messages directly via the main
    /// task's `Sender`. Otherwise you risk flooding the main thread with wakeup events.
    fn send(&mut self, msg: WindowEvent);
    /// Clones the main thread proxy.
    fn clone_main_thread_proxy(&self) -> Box<MainThreadProxy + 'static + Send>;
}

/// The port that the main thread receives window events on. As above, this is a trait supplied by
/// the Servo port.
pub trait MainThreadReceiver for Sized? : 'static {
    /// Receives the next event inbound for the main thread. This must not block.
    fn try_recv_main_thread_event(&mut self) -> Option<WindowEvent>;
    /// Synchronously waits for, and returns, the next message inbound for the main thread.
    fn recv_main_thread_event(&mut self) -> WindowEvent;
}

/// A convenience implementation of `MainThreadReceiver` for a plain old Rust `Receiver`.
impl MainThreadReceiver for Receiver<WindowEvent> {
    fn try_recv_main_thread_event(&mut self) -> Option<WindowEvent> {
        match self.try_recv() {
            Ok(msg) => Some(msg),
            Err(_) => None,
        }
    }
    fn recv_main_thread_event(&mut self) -> WindowEvent {
        self.recv()
    }
}

// FIXME(pcwalton): The double-indirection here is unfortunate.
impl ScriptToMainThreadProxy for Box<MainThreadProxy + Send> {
    fn quit(&mut self) {
        self.send(QuitWindowEvent)
    }
}

