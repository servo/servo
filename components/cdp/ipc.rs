// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.

use bincode;
use futures::{Async, Future};
use futures::future::{self, Either};
use ipc_channel::ipc::{self, IpcReceiver};
use msg::constellation_msg::{BrowsingContextId, TopLevelBrowsingContextId};
use script_traits::{ConstellationMsg, LoadData, WebDriverCommandMsg};
use script_traits::webdriver_msg::{LoadStatus, WebDriverScriptCommand};
use serde::{Deserialize, Serialize};
use servo_url::ServoUrl;
use std::io;
use std::sync::mpsc::Sender;

pub fn focus_top_level_browsing_context_id(
    constellation_chan: &Sender<ConstellationMsg>,
) -> io::Result<IpcReceiver<Option<TopLevelBrowsingContextId>>> {
    let (sender, receiver) = ipc::channel()?;
    let msg = ConstellationMsg::GetFocusTopLevelBrowsingContext(sender);
    send(constellation_chan, msg).map(|_| receiver)
}

pub fn get_page_title(
    constellation_chan: &Sender<ConstellationMsg>,
    browsing_context_id: BrowsingContextId,
) -> io::Result<IpcReceiver<String>> {
    let (sender, receiver) = ipc::channel()?;
    let msg = WebDriverScriptCommand::GetTitle(sender);
    send_webdriver_script_cmd(constellation_chan, browsing_context_id, msg).map(|_| receiver)
}

pub fn get_page_url(
    constellation_chan: &Sender<ConstellationMsg>,
    browsing_context_id: BrowsingContextId,
) -> io::Result<IpcReceiver<ServoUrl>> {
    let (sender, receiver) = ipc::channel()?;
    let msg = WebDriverScriptCommand::GetUrl(sender);
    send_webdriver_script_cmd(constellation_chan, browsing_context_id, msg).map(|_| receiver)
}

pub fn load_url(
    constellation_chan: &Sender<ConstellationMsg>,
    top_level_browsing_context_id: TopLevelBrowsingContextId,
    load_data: LoadData,
) -> io::Result<IpcReceiver<LoadStatus>> {
    let (sender, receiver) = ipc::channel()?;
    let msg = WebDriverCommandMsg::LoadUrl(top_level_browsing_context_id, load_data, sender);
    send_webdriver_cmd(constellation_chan, msg).map(|_| receiver)
}

fn send_webdriver_script_cmd(
    constellation_chan: &Sender<ConstellationMsg>,
    browsing_context_id: BrowsingContextId,
    msg: WebDriverScriptCommand,
) -> io::Result<()> {
    send_webdriver_cmd(
        constellation_chan,
        WebDriverCommandMsg::ScriptCommand(browsing_context_id, msg),
    )
}

fn send_webdriver_cmd(
    constellation_chan: &Sender<ConstellationMsg>,
    msg: WebDriverCommandMsg,
) -> io::Result<()> {
    send(constellation_chan, ConstellationMsg::WebDriverCommand(msg))
}

fn send<T>(sender: &Sender<T>, msg: T) -> io::Result<()> {
    sender.send(msg).map_err(|_| io::Error::new(io::ErrorKind::ConnectionReset, "channel closed"))
}

pub fn receive_ipc<T>(receiver: IpcReceiver<T>) -> impl Future<Item = T, Error = io::Error>
where
    for<'de> T: Serialize + Deserialize<'de>,
{
    future::poll_fn(move || match receiver.recv() {
        Ok(msg) => Ok(msg.into()),
        Err(err) => match *err {
            bincode::ErrorKind::IoError(ref e) if e.kind() == io::ErrorKind::WouldBlock => {
                Ok(Async::NotReady)
            }
            _ => Err(io::Error::new(io::ErrorKind::Other, err)),
        },
    })
}

pub fn wrap_receive_ipc<T>(
    result: io::Result<IpcReceiver<T>>,
) -> impl Future<Item = T, Error = io::Error>
where
    for<'de> T: Serialize + Deserialize<'de>,
{
    match result {
        Err(err) => Either::A(future::err(err)),
        Ok(receiver) => Either::B(receive_ipc(receiver)),
    }
}
