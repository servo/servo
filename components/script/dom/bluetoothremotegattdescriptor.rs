/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use bluetooth_blacklist::{Blacklist, uuid_is_blacklisted};
use dom::bindings::cell::DOMRefCell;
use dom::bindings::codegen::Bindings::BluetoothDeviceBinding::BluetoothDeviceMethods;
use dom::bindings::codegen::Bindings::BluetoothRemoteGATTCharacteristicBinding::
    BluetoothRemoteGATTCharacteristicMethods;
use dom::bindings::codegen::Bindings::BluetoothRemoteGATTDescriptorBinding;
use dom::bindings::codegen::Bindings::BluetoothRemoteGATTDescriptorBinding::BluetoothRemoteGATTDescriptorMethods;
use dom::bindings::codegen::Bindings::BluetoothRemoteGATTServerBinding::BluetoothRemoteGATTServerMethods;
use dom::bindings::codegen::Bindings::BluetoothRemoteGATTServiceBinding::BluetoothRemoteGATTServiceMethods;
use dom::bindings::error::Error::{self, InvalidModification, Network, Security};
use dom::bindings::global::GlobalRef;
use dom::bindings::js::{JS, MutHeap, Root};
use dom::bindings::refcounted::{Trusted, TrustedPromise};
use dom::bindings::reflector::{Reflectable, Reflector, reflect_dom_object};
use dom::bindings::str::{ByteString, DOMString};
use dom::bluetoothremotegattcharacteristic::{BluetoothRemoteGATTCharacteristic, MAXIMUM_ATTRIBUTE_LENGTH};
use dom::promise::Promise;
use ipc_channel::ipc::{self, IpcSender};
use ipc_channel::router::ROUTER;
use js::jsapi::JSAutoCompartment;
use net_traits::bluetooth_thread::{BluetoothMethodMsg, BluetoothResponseListener, BluetoothResultMsg};
use network_listener::{NetworkListener, PreInvoke};
use std::rc::Rc;
use std::sync::{Arc, Mutex};

struct BluetoothDescriptorContext {
    promise: Option<TrustedPromise>,
    descriptor: Trusted<BluetoothRemoteGATTDescriptor>,
}

// http://webbluetoothcg.github.io/web-bluetooth/#bluetoothremotegattdescriptor
#[dom_struct]
pub struct BluetoothRemoteGATTDescriptor {
    reflector_: Reflector,
    characteristic: MutHeap<JS<BluetoothRemoteGATTCharacteristic>>,
    uuid: DOMString,
    value: DOMRefCell<Option<ByteString>>,
    instance_id: String,
}

impl BluetoothRemoteGATTDescriptor {
    pub fn new_inherited(characteristic: &BluetoothRemoteGATTCharacteristic,
                         uuid: DOMString,
                         instance_id: String)
                         -> BluetoothRemoteGATTDescriptor {
        BluetoothRemoteGATTDescriptor {
            reflector_: Reflector::new(),
            characteristic: MutHeap::new(characteristic),
            uuid: uuid,
            value: DOMRefCell::new(None),
            instance_id: instance_id,
        }
    }

    pub fn new(global: GlobalRef,
               characteristic: &BluetoothRemoteGATTCharacteristic,
               uuid: DOMString,
               instanceID: String)
               -> Root<BluetoothRemoteGATTDescriptor>{
        reflect_dom_object(box BluetoothRemoteGATTDescriptor::new_inherited(characteristic,
                                                                            uuid,
                                                                            instanceID),
                            global,
                            BluetoothRemoteGATTDescriptorBinding::Wrap)
    }

    fn get_bluetooth_thread(&self) -> IpcSender<BluetoothMethodMsg> {
        let global_root = self.global();
        let global_ref = global_root.r();
        global_ref.as_window().bluetooth_thread()
    }

    fn get_instance_id(&self) -> String {
        self.instance_id.clone()
    }
}

impl BluetoothRemoteGATTDescriptorMethods for BluetoothRemoteGATTDescriptor {
    // https://webbluetoothcg.github.io/web-bluetooth/#dom-bluetoothremotegattdescriptor-characteristic
    fn Characteristic(&self) -> Root<BluetoothRemoteGATTCharacteristic> {
       self.characteristic.get()
    }

    // https://webbluetoothcg.github.io/web-bluetooth/#dom-bluetoothremotegattdescriptor-uuid
    fn Uuid(&self) -> DOMString {
        self.uuid.clone()
    }

     // https://webbluetoothcg.github.io/web-bluetooth/#dom-bluetoothremotegattdescriptor-value
    fn GetValue(&self) -> Option<ByteString> {
        self.value.borrow().clone()
    }

    #[allow(unrooted_must_root)]
    // https://webbluetoothcg.github.io/web-bluetooth/#dom-bluetoothremotegattdescriptor-readvalue
    fn ReadValue(&self) -> Rc<Promise> {
        let p = Promise::new(self.global().r());
        let p_cx = p.global().r().get_cx();
        if uuid_is_blacklisted(self.uuid.as_ref(), Blacklist::Reads) {
            p.reject_error(p_cx, Security);
            return p;
        }
        if !self.Characteristic().Service().Device().Gatt().Connected() {
            p.reject_error(p_cx, Network);
            return p;
        }
        let (sender, receiver) = ipc::channel().unwrap();
        let btd_context = Arc::new(Mutex::new(BluetoothDescriptorContext {
            promise: Some(TrustedPromise::new(p.clone())),
            descriptor: Trusted::new(self),
        }));
        let listener = NetworkListener {
            context: btd_context,
            script_chan: self.global().r().networking_task_source(),
            wrapper: None,
        };
        ROUTER.add_route(receiver.to_opaque(), box move |message| {
            listener.notify_response(message.to().unwrap());
        });
        self.get_bluetooth_thread().send(
            BluetoothMethodMsg::ReadValue(self.get_instance_id(), sender)).unwrap();
        return p;
    }

    #[allow(unrooted_must_root)]
    // https://webbluetoothcg.github.io/web-bluetooth/#dom-bluetoothremotegattdescriptor-writevalue
    fn WriteValue(&self, value: Vec<u8>) -> Rc<Promise> {
        let p = Promise::new(self.global().r());
        let p_cx = p.global().r().get_cx();
        if uuid_is_blacklisted(self.uuid.as_ref(), Blacklist::Writes) {
            p.reject_error(p_cx, Security);
            return p;
        }
        if value.len() > MAXIMUM_ATTRIBUTE_LENGTH {
            p.reject_error(p_cx, InvalidModification);
            return p;
        }
        if !self.Characteristic().Service().Device().Gatt().Connected() {
            p.reject_error(p_cx, Network);
            return p;
        }
        let (sender, receiver) = ipc::channel().unwrap();
        let btd_context = Arc::new(Mutex::new(BluetoothDescriptorContext {
            promise: Some(TrustedPromise::new(p.clone())),
            descriptor: Trusted::new(self),
        }));
        let listener = NetworkListener {
            context: btd_context,
            script_chan: self.global().r().networking_task_source(),
            wrapper: None,
        };
        ROUTER.add_route(receiver.to_opaque(), box move |message| {
            listener.notify_response(message.to().unwrap());
        });
        self.get_bluetooth_thread().send(
            BluetoothMethodMsg::WriteValue(self.get_instance_id(), value, sender)).unwrap();
        return p;
    }
}

impl PreInvoke for BluetoothDescriptorContext {}

impl BluetoothResponseListener for BluetoothDescriptorContext {
    #[allow(unrooted_must_root)]
    fn response(&mut self, result: BluetoothResultMsg) {
        let promise = self.promise.take().expect("bt promise is missing").root();
        let promise_cx = promise.global().r().get_cx();

        // JSAutoCompartment needs to be manually made.
        // Otherwise, Servo will crash.
        let _ac = JSAutoCompartment::new(promise_cx, promise.reflector().get_jsobject().get());
        match result {
            BluetoothResultMsg::ReadValue(result) => {
                let value = ByteString::new(result);
                let d = self.descriptor.root();
                *d.value.borrow_mut() = Some(value.clone());
                promise.resolve_native(
                    promise_cx,
                    &value);
            },
            BluetoothResultMsg::WriteValue(result) => {
                promise.resolve_native(
                    promise_cx,
                    &result);
            },
            BluetoothResultMsg::Error(error) => {
                promise.reject_error(
                    promise_cx,
                    Error::from(error));
            },
            _ => {
                promise.reject_error(
                    promise_cx,
                    Error::Type("Something went wrong...".to_owned()));
            }
        }
        self.promise = Some(TrustedPromise::new(promise));
    }
}
