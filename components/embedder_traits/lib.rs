/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

#[macro_use] 
extern crate lazy_static;
#[macro_use]
extern crate log;
#[macro_use]
extern crate serde;

extern crate ipc_channel;
extern crate msg;
extern crate servo_url;
extern crate style_traits;
extern crate webrender_api;

pub mod resources;

use ipc_channel::ipc::IpcSender;
use msg::constellation_msg::{InputMethodType, Key, KeyModifiers, KeyState, TopLevelBrowsingContextId};
use servo_url::ServoUrl;
use std::fmt::{Debug, Error, Formatter};
use std::sync::mpsc::{Receiver, Sender};
use style_traits::cursor::CursorKind;
use webrender_api::{DeviceIntPoint, DeviceUintSize};


/// Used to wake up the event loop, provided by the servo port/embedder.
pub trait EventLoopWaker : 'static + Send {
    fn clone(&self) -> Box<EventLoopWaker + Send>;
    fn wake(&self);
}

/// Sends messages to the embedder.
pub struct EmbedderProxy {
    pub sender: Sender<EmbedderMsg>,
    pub event_loop_waker: Box<EventLoopWaker>,
}

impl EmbedderProxy {
    pub fn send(&self, msg: EmbedderMsg) {
        // Send a message and kick the OS event loop awake.
        if let Err(err) = self.sender.send(msg) {
            warn!("Failed to send response ({}).", err);
        }
        self.event_loop_waker.wake();
    }
}

impl Clone for EmbedderProxy {
    fn clone(&self) -> EmbedderProxy {
        EmbedderProxy {
            sender: self.sender.clone(),
            event_loop_waker: self.event_loop_waker.clone(),
        }
    }
}

/// The port that the embedder receives messages on.
pub struct EmbedderReceiver {
    pub receiver: Receiver<EmbedderMsg>
}

impl EmbedderReceiver {
    pub fn try_recv_embedder_msg(&mut self) -> Option<EmbedderMsg> {
        self.receiver.try_recv().ok()
    }
    pub fn recv_embedder_msg(&mut self) -> EmbedderMsg {
        self.receiver.recv().unwrap()
    }
}

#[derive(Deserialize, Serialize)]
pub enum EmbedderMsg {
    /// A status message to be displayed by the browser chrome.
    Status(TopLevelBrowsingContextId, Option<String>),
    /// Alerts the embedder that the current page has changed its title.
    ChangePageTitle(TopLevelBrowsingContextId, Option<String>),
    /// Move the window to a point
    MoveTo(TopLevelBrowsingContextId, DeviceIntPoint),
    /// Resize the window to size
    ResizeTo(TopLevelBrowsingContextId, DeviceUintSize),
    /// Handle an alert, and respond on whether script should show one.
    Alert(TopLevelBrowsingContextId, String, IpcSender<bool>),
    /// Wether or not to follow a link
    AllowNavigation(TopLevelBrowsingContextId, ServoUrl, IpcSender<bool>),
    /// Sends an unconsumed key event back to the embedder.
    KeyEvent(Option<TopLevelBrowsingContextId>, Option<char>, Key, KeyState, KeyModifiers),
    /// Changes the cursor.
    SetCursor(CursorKind),
    /// A favicon was detected
    NewFavicon(TopLevelBrowsingContextId, ServoUrl),
    /// <head> tag finished parsing
    HeadParsed(TopLevelBrowsingContextId),
    /// The history state has changed.
    HistoryChanged(TopLevelBrowsingContextId, Vec<ServoUrl>, usize),
    /// Enter or exit fullscreen
    SetFullscreenState(TopLevelBrowsingContextId, bool),
    /// The load of a page has begun
    LoadStart(TopLevelBrowsingContextId),
    /// The load of a page has completed
    LoadComplete(TopLevelBrowsingContextId),
    /// A pipeline panicked. First string is the reason, second one is the backtrace.
    Panic(TopLevelBrowsingContextId, String, Option<String>),
    /// Open dialog to select bluetooth device.
    GetSelectedBluetoothDevice(Vec<String>, IpcSender<Option<String>>),
    /// Request to present an IME to the user when an editable element is focused.
    ShowIME(TopLevelBrowsingContextId, InputMethodType),
    /// Request to hide the IME when the editable element is blurred.
    HideIME(TopLevelBrowsingContextId),
    /// Servo has shut down
    Shutdown,
}

impl Debug for EmbedderMsg {
    fn fmt(&self, f: &mut Formatter) -> Result<(), Error> {
        match *self {
            EmbedderMsg::Status(..) => write!(f, "Status"),
            EmbedderMsg::ChangePageTitle(..) => write!(f, "ChangePageTitle"),
            EmbedderMsg::MoveTo(..) => write!(f, "MoveTo"),
            EmbedderMsg::ResizeTo(..) => write!(f, "ResizeTo"),
            EmbedderMsg::Alert(..) => write!(f, "Alert"),
            EmbedderMsg::AllowNavigation(..) => write!(f, "AllowNavigation"),
            EmbedderMsg::KeyEvent(..) => write!(f, "KeyEvent"),
            EmbedderMsg::SetCursor(..) => write!(f, "SetCursor"),
            EmbedderMsg::NewFavicon(..) => write!(f, "NewFavicon"),
            EmbedderMsg::HeadParsed(..) => write!(f, "HeadParsed"),
            EmbedderMsg::HistoryChanged(..) => write!(f, "HistoryChanged"),
            EmbedderMsg::SetFullscreenState(..) => write!(f, "SetFullscreenState"),
            EmbedderMsg::LoadStart(..) => write!(f, "LoadStart"),
            EmbedderMsg::LoadComplete(..) => write!(f, "LoadComplete"),
            EmbedderMsg::Panic(..) => write!(f, "Panic"),
            EmbedderMsg::GetSelectedBluetoothDevice(..) => write!(f, "GetSelectedBluetoothDevice"),
            EmbedderMsg::ShowIME(..) => write!(f, "ShowIME"),
            EmbedderMsg::HideIME(..) => write!(f, "HideIME"),
            EmbedderMsg::Shutdown => write!(f, "Shutdown"),
        }
    }
}
