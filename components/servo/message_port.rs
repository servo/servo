/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use std::cell::RefCell;
use std::rc::Rc;

use base::id::{MessagePortId, MessagePortRouterId, WebViewId};
use constellation_traits::{EmbedderToConstellationMessage, MessagePortMsg, PostMessageData};
use embedder_traits::{EmbedderMsg, EmbedderProxy, JSValue};
use ipc_channel::ipc;
use ipc_channel::router::ROUTER;

use crate::{ConstellationProxy, WebView};

pub trait MessagePortDelegate {
    /// Invoked when a message is posted on an entangled MessagePort. The
    /// posted data is provided as an argument, and only includes seralized
    /// versions of fundamental JS data types.
    fn post_message_received(&self, _webview: WebView, _message_port: MessagePort, _data: JSValue) {
    }
}

enum MessagePortState {
    Ready {
        id: MessagePortId,
        webview_id: WebViewId,
        entangled_id: MessagePortId,
        constellation_proxy: ConstellationProxy,
        delegate: Option<Rc<dyn MessagePortDelegate>>,
    },
    Detached,
}

#[derive(Clone)]
pub struct MessagePort(Rc<RefCell<MessagePortState>>);

#[derive(Debug, PartialEq)]
pub struct MessagePortDetached;

impl MessagePort {
    fn new(
        constellation_proxy: ConstellationProxy,
        webview_id: WebViewId,
        port_id: MessagePortId,
        entangled_id: MessagePortId,
    ) -> MessagePort {
        MessagePort(Rc::new(RefCell::new(MessagePortState::Ready {
            constellation_proxy,
            webview_id,
            id: port_id,
            entangled_id,
            delegate: None,
        })))
    }

    pub(crate) fn new_entangled(
        constellation_proxy: ConstellationProxy,
        embedder_proxy: EmbedderProxy,
        webview_id: WebViewId,
    ) -> (MessagePort, MessagePort) {
        let (port_control_sender, port_control_receiver) =
            ipc::channel().expect("ipc channel failure");
        ROUTER.add_typed_route(
            port_control_receiver,
            Box::new(move |message| match message.unwrap() {
                MessagePortMsg::CompleteTransfer(..) => unreachable!(),
                MessagePortMsg::CompletePendingTransfer(..) => unreachable!(),
                MessagePortMsg::CompleteDisentanglement(..) => unreachable!(),
                MessagePortMsg::NewTask(message_port_id, task) => {
                    let data = match task.data {
                        PostMessageData::StructuredClone(..) => unreachable!(),
                        PostMessageData::Serialized(data) => data,
                    };
                    let _ = embedder_proxy.send(EmbedderMsg::PostMessage(
                        webview_id,
                        message_port_id,
                        data,
                    ));
                },
            }),
        );
        let first_id = MessagePortId::new();
        let second_id = MessagePortId::new();
        let router_id = MessagePortRouterId::new();
        constellation_proxy.send(EmbedderToConstellationMessage::CreateEntangledMessagePorts(
            router_id,
            port_control_sender,
            first_id,
            second_id,
        ));

        let first = MessagePort::new(constellation_proxy.clone(), webview_id, first_id, second_id);
        let second = MessagePort::new(constellation_proxy, webview_id, second_id, first_id);

        (first, second)
    }

    /// Post the provided [JSValue] to the MessagePort that is entangled with this one.
    pub fn post_message(&self, js_value: JSValue) -> Result<(), MessagePortDetached> {
        let inner = self.0.borrow();
        let MessagePortState::Ready {
            entangled_id,
            constellation_proxy,
            ..
        } = &*inner
        else {
            return Err(MessagePortDetached);
        };
        constellation_proxy.send(EmbedderToConstellationMessage::PostMessageToMessagePort(
            *entangled_id,
            js_value,
        ));
        Ok(())
    }

    /// Set the delegate of this MessagePort. Does nothing if this MessagePort
    /// has already been transferred.
    pub fn set_delegate(&self, new_delegate: Rc<dyn MessagePortDelegate>) {
        match &mut *self.0.borrow_mut() {
            MessagePortState::Ready { delegate, .. } => *delegate = Some(new_delegate),
            MessagePortState::Detached => {},
        }
    }

    pub fn delegate(&self) -> Option<Rc<dyn MessagePortDelegate>> {
        match &*self.0.borrow() {
            MessagePortState::Ready { delegate, .. } => delegate.clone(),
            MessagePortState::Detached => None,
        }
    }

    /// Prepare this MessagePort to be transferred as part of a PostMessage operation.
    /// This object will no longer be usable afterwards.
    pub fn into_jsvalue(&self) -> Result<JSValue, MessagePortDetached> {
        let value = {
            let state = self.0.borrow();
            let &MessagePortState::Ready {
                webview_id,
                id,
                entangled_id,
                ..
            } = &*state
            else {
                return Err(MessagePortDetached);
            };
            JSValue::MessagePort {
                webview_id,
                id,
                entangled: entangled_id,
            }
        };
        *self.0.borrow_mut() = MessagePortState::Detached;
        Ok(value)
    }

    pub(crate) fn id(&self) -> Option<MessagePortId> {
        match &*self.0.borrow() {
            MessagePortState::Ready { id, .. } => Some(*id),
            MessagePortState::Detached => None,
        }
    }
}
