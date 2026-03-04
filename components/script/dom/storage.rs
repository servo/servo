/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */
use base::generic_channel::{GenericSend, GenericSender, ReceiveResult, SendResult};
use base::id::WebViewId;
use constellation_traits::ScriptToConstellationMessage;
use dom_struct::dom_struct;
use profile_traits::generic_channel;
use profile_traits::generic_channel::GenericReceiver;
use serde::{Deserialize, Serialize};
use servo_url::ServoUrl;
use storage_traits::webstorage_thread::{WebStorageThreadMsg, WebStorageType};

use crate::dom::bindings::codegen::Bindings::StorageBinding::StorageMethods;
use crate::dom::bindings::error::{Error, ErrorResult};
use crate::dom::bindings::inheritance::Castable;
use crate::dom::bindings::refcounted::Trusted;
use crate::dom::bindings::reflector::{DomGlobal, Reflector, reflect_dom_object};
use crate::dom::bindings::root::DomRoot;
use crate::dom::bindings::str::DOMString;
use crate::dom::event::{Event, EventBubbles, EventCancelable};
use crate::dom::storageevent::StorageEvent;
use crate::dom::window::Window;
use crate::script_runtime::CanGc;

#[dom_struct]
pub(crate) struct Storage {
    reflector_: Reflector,
    #[no_trace]
    storage_type: WebStorageType,
}

impl Storage {
    fn new_inherited(storage_type: WebStorageType) -> Storage {
        Storage {
            reflector_: Reflector::new(),
            storage_type,
        }
    }

    pub(crate) fn new(
        global: &Window,
        storage_type: WebStorageType,
        can_gc: CanGc,
    ) -> DomRoot<Storage> {
        reflect_dom_object(
            Box::new(Storage::new_inherited(storage_type)),
            global,
            can_gc,
        )
    }

    fn webview_id(&self) -> WebViewId {
        self.global().as_window().window_proxy().webview_id()
    }

    fn get_url(&self) -> ServoUrl {
        self.global().get_url()
    }

    fn send_msg(&self, msg: WebStorageThreadMsg, error_msg: &str) -> SendResult {
        GenericSend::send(self.global().storage_threads(), msg)
            .inspect_err(|err| error!("Failed to send WebStorageThreadMsg {error_msg}: {err}"))
    }

    fn recv_msg<T>(receiver: GenericReceiver<T>, error_msg: &str) -> ReceiveResult<T>
    where
        T: Serialize + for<'de> Deserialize<'de>,
    {
        receiver
            .recv()
            .inspect_err(|err| error!("Failed to receive response for {error_msg}: {err:?}"))
    }

    fn channel<T>(&self) -> (GenericSender<T>, GenericReceiver<T>)
    where
        T: Serialize + for<'de> Deserialize<'de>,
    {
        generic_channel::channel(self.global().time_profiler_chan().clone())
            .expect("Failed to create channel")
    }
}

impl StorageMethods<crate::DomTypeHolder> for Storage {
    /// <https://html.spec.whatwg.org/multipage/#dom-storage-length>
    fn Length(&self) -> u32 {
        let (sender, receiver) = self.channel();

        let msg = WebStorageThreadMsg::Length(
            sender,
            self.storage_type,
            self.webview_id(),
            self.get_url(),
        );
        if self.send_msg(msg, "Length").is_err() {
            return 0;
        }
        Self::recv_msg(receiver, "Length").unwrap_or_default() as u32
    }

    /// <https://html.spec.whatwg.org/multipage/#dom-storage-key>
    fn Key(&self, index: u32) -> Option<DOMString> {
        let (sender, receiver) = self.channel();

        let msg = WebStorageThreadMsg::Key(
            sender,
            self.storage_type,
            self.webview_id(),
            self.get_url(),
            index,
        );
        self.send_msg(msg, "Key").ok()?;
        Self::recv_msg(receiver, "Key").ok()?.map(DOMString::from)
    }

    /// <https://html.spec.whatwg.org/multipage/#dom-storage-getitem>
    fn GetItem(&self, name: DOMString) -> Option<DOMString> {
        let (sender, receiver) = self.channel();

        let msg = WebStorageThreadMsg::GetItem(
            sender,
            self.storage_type,
            self.webview_id(),
            self.get_url(),
            String::from(name),
        );
        self.send_msg(msg, "GetItem").ok()?;
        Self::recv_msg(receiver, "GetItem")
            .ok()?
            .map(DOMString::from)
    }

    /// <https://html.spec.whatwg.org/multipage/#dom-storage-setitem>
    fn SetItem(&self, name: DOMString, value: DOMString) -> ErrorResult {
        let (sender, receiver) = self.channel();
        let name = String::from(name);
        let value = String::from(value);

        let msg = WebStorageThreadMsg::SetItem(
            sender,
            self.storage_type,
            self.webview_id(),
            self.get_url(),
            name.clone(),
            value.clone(),
        );
        self.send_msg(msg, "SetItem")
            .map_err(|_| Error::Operation(Some("Failed to send SetItem msg".into())))?;
        match Self::recv_msg(receiver, "SetItem")
            .map_err(|_| Error::Operation(Some("Failed to recv SetItem response".into())))?
        {
            Err(_) => Err(Error::QuotaExceeded {
                quota: None,
                requested: None,
            }),
            Ok((changed, old_value)) => {
                if changed {
                    self.broadcast_change_notification(Some(name), old_value, Some(value));
                }
                Ok(())
            },
        }
    }

    /// <https://html.spec.whatwg.org/multipage/#dom-storage-removeitem>
    fn RemoveItem(&self, name: DOMString) {
        let (sender, receiver) = self.channel();
        let name = String::from(name);

        let msg = WebStorageThreadMsg::RemoveItem(
            sender,
            self.storage_type,
            self.webview_id(),
            self.get_url(),
            name.clone(),
        );
        if self.send_msg(msg, "RemoveItem").is_ok() {
            if let Ok(Some(old_value)) = Self::recv_msg(receiver, "RemoveItem") {
                self.broadcast_change_notification(Some(name), Some(old_value), None);
            }
        }
    }

    /// <https://html.spec.whatwg.org/multipage/#dom-storage-clear>
    fn Clear(&self) {
        let (sender, receiver) = self.channel();

        let msg = WebStorageThreadMsg::Clear(
            sender,
            self.storage_type,
            self.webview_id(),
            self.get_url(),
        );
        if self.send_msg(msg, "Clear").is_err() {
            return;
        }
        if Self::recv_msg(receiver, "Clear").unwrap_or_default() {
            self.broadcast_change_notification(None, None, None);
        }
    }

    /// <https://html.spec.whatwg.org/multipage/#the-storage-interface:supported-property-names>
    fn SupportedPropertyNames(&self) -> Vec<DOMString> {
        let (sender, receiver) = self.channel();

        let msg =
            WebStorageThreadMsg::Keys(sender, self.storage_type, self.webview_id(), self.get_url());
        if self.send_msg(msg, "Keys").is_err() {
            return vec![];
        }
        Self::recv_msg(receiver, "Keys")
            .unwrap_or_default()
            .into_iter()
            .map(DOMString::from)
            .collect()
    }

    // check-tidy: no specs after this line
    fn NamedGetter(&self, name: DOMString) -> Option<DOMString> {
        self.GetItem(name)
    }

    fn NamedSetter(&self, name: DOMString, value: DOMString) -> ErrorResult {
        self.SetItem(name, value)
    }

    fn NamedDeleter(&self, name: DOMString) {
        self.RemoveItem(name);
    }
}

impl Storage {
    /// <https://html.spec.whatwg.org/multipage/#send-a-storage-notification>
    fn broadcast_change_notification(
        &self,
        key: Option<String>,
        old_value: Option<String>,
        new_value: Option<String>,
    ) {
        let storage = self.storage_type;
        let url = self.get_url();
        let msg = ScriptToConstellationMessage::BroadcastStorageEvent(
            storage, url, key, old_value, new_value,
        );
        let _ = self.global().script_to_constellation_chan().send(msg);
    }

    /// <https://html.spec.whatwg.org/multipage/#send-a-storage-notification>
    pub(crate) fn queue_storage_event(
        &self,
        url: ServoUrl,
        key: Option<String>,
        old_value: Option<String>,
        new_value: Option<String>,
    ) {
        let global = self.global();
        let this = Trusted::new(self);
        global.task_manager().dom_manipulation_task_source().queue(
            task!(send_storage_notification: move || {
                let this = this.root();
                let global = this.global();
                let event = StorageEvent::new(
                    global.as_window(),
                    atom!("storage"),
                    EventBubbles::DoesNotBubble,
                    EventCancelable::NotCancelable,
                    key.map(DOMString::from),
                    old_value.map(DOMString::from),
                    new_value.map(DOMString::from),
                    DOMString::from(url.into_string()),
                    Some(&this),
                    CanGc::note()
                );
                event.upcast::<Event>().fire(global.upcast(), CanGc::note());
            }),
        );
    }
}
