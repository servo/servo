/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use msg::constellation_msg::ConstellationChan;
use msg::constellation_msg::Msg as ConstellationMsg;

use ipc_channel::ipc;
use std::borrow::ToOwned;
use std::sync::mpsc::channel;

pub trait ClipboardProvider {
    // blocking method to get the clipboard contents
    fn clipboard_contents(&mut self) -> String;
    // blocking method to set the clipboard contents
    fn set_clipboard_contents(&mut self, String);
}

impl ClipboardProvider for ConstellationChan {
    fn clipboard_contents(&mut self) -> String {
        let (tx, rx) = ipc::channel().unwrap();
        self.0.send(ConstellationMsg::GetClipboardContents(tx)).unwrap();
        rx.recv().unwrap()
    }
    fn set_clipboard_contents(&mut self, s: String) {
        self.0.send(ConstellationMsg::SetClipboardContents(s)).unwrap();
    }
}

pub struct DummyClipboardContext {
    content: String
}

impl DummyClipboardContext {
    pub fn new(s: &str) -> DummyClipboardContext {
        DummyClipboardContext {
            content: s.to_owned()
        }
    }
}

impl ClipboardProvider for DummyClipboardContext {
    fn clipboard_contents(&mut self) -> String {
        self.content.clone()
    }
    fn set_clipboard_contents(&mut self, s: String) {
        self.content = s;
    }
}
