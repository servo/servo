/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

extern crate crossbeam_channel;
extern crate ipc_channel;
#[macro_use]
extern crate lazy_static;
extern crate msg;
#[macro_use]
extern crate serde;
extern crate servo_url;
extern crate style_traits;
extern crate webrender_api;

pub mod resources;

use crossbeam_channel::{Receiver, Sender};
use ipc_channel::ipc::IpcSender;
use msg::constellation_msg::{InputMethodType, Key, KeyModifiers, KeyState, TopLevelBrowsingContextId};
use servo_url::ServoUrl;
use std::fmt::{Debug, Error, Formatter};
use style_traits::cursor::CursorKind;
use webrender_api::{DeviceIntPoint, DeviceUintSize};


/// Used to wake up the event loop, provided by the servo port/embedder.
pub trait EventLoopWaker : 'static + Send {
    fn clone(&self) -> Box<EventLoopWaker + Send>;
    fn wake(&self);
}

/// Sends messages to the embedder.
pub struct EmbedderProxy {
    pub sender: Sender<(Option<TopLevelBrowsingContextId>, EmbedderMsg)>,
    pub event_loop_waker: Box<EventLoopWaker>,
}

impl EmbedderProxy {
    pub fn send(&self, msg: (Option<TopLevelBrowsingContextId>, EmbedderMsg)) {
        self.sender.send(msg);
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
    pub receiver: Receiver<(Option<TopLevelBrowsingContextId>, EmbedderMsg)>
}

impl EmbedderReceiver {
    pub fn try_recv_embedder_msg(&mut self) -> Option<(Option<TopLevelBrowsingContextId>, EmbedderMsg)> {
        self.receiver.try_recv()
    }
    pub fn recv_embedder_msg(&mut self) -> (Option<TopLevelBrowsingContextId>, EmbedderMsg) {
        self.receiver.recv().unwrap()
    }
}

#[derive(Deserialize, Serialize)]
pub enum EmbedderMsg {
    /// A status message to be displayed by the browser chrome.
    Status(Option<String>),
    /// Alerts the embedder that the current page has changed its title.
    ChangePageTitle(Option<String>),
    /// Move the window to a point
    MoveTo(DeviceIntPoint),
    /// Resize the window to size
    ResizeTo(DeviceUintSize),
    // Show an alert message.
    Alert(String, IpcSender<()>),
    /// Wether or not to follow a link
    AllowNavigation(ServoUrl, IpcSender<bool>),
    /// Wether or not to unload a document
    AllowUnload(IpcSender<bool>),
    /// Sends an unconsumed key event back to the embedder.
    KeyEvent(Option<char>, Key, KeyState, KeyModifiers),
    /// Changes the cursor.
    SetCursor(CursorKind),
    /// A favicon was detected
    NewFavicon(ServoUrl),
    /// <head> tag finished parsing
    HeadParsed,
    /// The history state has changed.
    HistoryChanged(Vec<ServoUrl>, usize),
    /// Enter or exit fullscreen
    SetFullscreenState(bool),
    /// The load of a page has begun
    LoadStart,
    /// The load of a page has completed
    LoadComplete,
    /// A browser is to be closed
    CloseBrowser,
    /// A pipeline panicked. First string is the reason, second one is the backtrace.
    Panic(String, Option<String>),
    /// Open dialog to select bluetooth device.
    GetSelectedBluetoothDevice(Vec<String>, IpcSender<Option<String>>),
    /// Open file dialog to select files. Set boolean flag to true allows to select multiple files.
    SelectFiles(Vec<FilterPattern>, bool, IpcSender<Option<Vec<String>>>),
    /// Request to present an IME to the user when an editable element is focused.
    ShowIME(InputMethodType),
    /// Request to hide the IME when the editable element is blurred.
    HideIME,
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
            EmbedderMsg::AllowUnload(..) => write!(f, "AllowUnload"),
            EmbedderMsg::AllowNavigation(..) => write!(f, "AllowNavigation"),
            EmbedderMsg::KeyEvent(..) => write!(f, "KeyEvent"),
            EmbedderMsg::SetCursor(..) => write!(f, "SetCursor"),
            EmbedderMsg::NewFavicon(..) => write!(f, "NewFavicon"),
            EmbedderMsg::HeadParsed => write!(f, "HeadParsed"),
            EmbedderMsg::CloseBrowser => write!(f, "CloseBrowser"),
            EmbedderMsg::HistoryChanged(..) => write!(f, "HistoryChanged"),
            EmbedderMsg::SetFullscreenState(..) => write!(f, "SetFullscreenState"),
            EmbedderMsg::LoadStart => write!(f, "LoadStart"),
            EmbedderMsg::LoadComplete => write!(f, "LoadComplete"),
            EmbedderMsg::Panic(..) => write!(f, "Panic"),
            EmbedderMsg::GetSelectedBluetoothDevice(..) => write!(f, "GetSelectedBluetoothDevice"),
            EmbedderMsg::SelectFiles(..) => write!(f, "SelectFiles"),
            EmbedderMsg::ShowIME(..) => write!(f, "ShowIME"),
            EmbedderMsg::HideIME => write!(f, "HideIME"),
            EmbedderMsg::Shutdown => write!(f, "Shutdown"),
        }
    }
}

/// Filter for file selection;
/// the `String` content is expected to be extension (e.g, "doc", without the prefixing ".")
#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct FilterPattern(pub String);
