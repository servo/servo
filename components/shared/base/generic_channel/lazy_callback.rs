/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

//! # Lazy Callbacks
//!
//! When constructing callbacks we sometimes have a large distance between where the callback is setup and where
//! the initial callback will be called. Refactoring of this code is sometimes not possible.
//! Here we provide [LazyCallback]. We use 'lazy_callback()' to generate a [LazyCallback] and a [CallbackSetter].
//! The [LazyCallback] works like a [GenericCallback] and can be used to send messages.
//! The [CallbackSetter] has a single consuming method of 'set_callback' which will set the callback that the [LazyCallback]
//! will then execute on messages send to it.
//!
//! This is achieved with having the LazyCallback having a back channel in single process mode that sets the [GenericCallback].
//! Hence, this is slightly less efficient than a [GenericCallback]

use std::cell::{OnceCell, RefCell};
use std::fmt;
use std::marker::PhantomData;

use ipc_channel::ErrorKind;
use ipc_channel::ipc::{IpcReceiver, IpcSender};
use ipc_channel::router::ROUTER;
use serde::de::VariantAccess;
use serde::{Deserialize, Deserializer, Serialize, Serializer};
use servo_config::opts;

use crate::generic_channel::{GenericCallback, SendError, SendResult};

/// Basic struct for [LazyCallback]
pub struct LazyCallback<T: Serialize + for<'de> Deserialize<'de> + Send + 'static>(
    LazyCallbackVariants<T>,
);

enum LazyCallbackVariants<T>
where
    T: Serialize + Send + 'static,
{
    InProcess {
        callback_receiver: RefCell<Option<crossbeam_channel::Receiver<GenericCallback<T>>>>,
        callback: OnceCell<GenericCallback<T>>,
    },
    Ipc(IpcSender<T>),
}

impl<T> LazyCallback<T>
where
    T: Serialize + for<'de> Deserialize<'de> + Send + 'static,
{
    /// Send messages to the callback. This might block until the callback is set via the 'CallbackSetter'
    pub fn send(&self, value: T) -> SendResult {
        match &self.0 {
            LazyCallbackVariants::InProcess {
                callback_receiver,
                callback,
            } => {
                let cb = callback.get_or_init(|| {
                    callback_receiver
                        .borrow_mut()
                        .take()
                        .unwrap()
                        .recv()
                        .unwrap()
                });
                cb.send(value)
            },
            LazyCallbackVariants::Ipc(ipc_sender) => {
                ipc_sender.send(value).map_err(|error| match *error {
                    ErrorKind::Io(_) => SendError::Disconnected,
                    serialization_error => {
                        SendError::SerializationError(serialization_error.to_string())
                    },
                })
            },
        }
    }
}

pub struct CallbackSetter<T: Serialize + Send + 'static>(CallbackSetterVariants<T>);

impl<T: Serialize + Send> fmt::Debug for CallbackSetter<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_tuple("CallbackSetter").finish()
    }
}

impl<T> Serialize for CallbackSetter<T>
where
    T: Serialize + Send + 'static,
{
    fn serialize<S: Serializer>(&self, s: S) -> Result<S::Ok, S::Error> {
        match &self.0 {
            CallbackSetterVariants::Ipc(sender) => {
                s.serialize_newtype_variant("CallbackSetter", 0, "Ipc", sender)
            },
            // The only reason we need / want serialization in single-process mode is to support
            // sending GenericCallbacks over existing IPC channels. This allows us to
            // incrementally port IPC channels to the GenericChannel, without needing to follow a
            // top-to-bottom approach.
            // Long-term we can remove this branch in the code again and replace it with
            // unreachable, since likely all IPC channels would be GenericChannels.
            CallbackSetterVariants::InProcess(wrapped_callback) => {
                if opts::get().multiprocess {
                    return Err(serde::ser::Error::custom(
                        "InProcess callback setter can't be serialized in multiprocess mode",
                    ));
                }
                // Due to the signature of `serialize` we need to clone the Arc to get an owned
                // pointer we can leak.
                // We additionally need to Box to get a thin pointer.
                let cloned_callback = Box::new(wrapped_callback.clone());
                let sender_clone_addr = Box::leak(cloned_callback) as *mut _ as usize;
                s.serialize_newtype_variant("CallbackSetter", 1, "InProcess", &sender_clone_addr)
            },
        }
    }
}

struct LazyCallbackSetterVisitor<T> {
    marker: PhantomData<T>,
}

impl<'de, T> serde::de::Visitor<'de> for LazyCallbackSetterVisitor<T>
where
    T: Serialize + Deserialize<'de> + Send + 'static,
{
    type Value = CallbackSetter<T>;

    fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        formatter.write_str("a GenericCallback variant")
    }

    fn visit_enum<A>(self, data: A) -> Result<Self::Value, A::Error>
    where
        A: serde::de::EnumAccess<'de>,
    {
        #[derive(Deserialize)]
        enum LazyCallbackSetterVariantNames {
            Ipc,
            InProcess,
        }

        let (variant_name, variant_data): (LazyCallbackSetterVariantNames, _) = data.variant()?;

        match variant_name {
            LazyCallbackSetterVariantNames::Ipc => variant_data
                .newtype_variant::<IpcReceiver<T>>()
                .map(|receiver| CallbackSetter(CallbackSetterVariants::Ipc(receiver))),
            LazyCallbackSetterVariantNames::InProcess => {
                if opts::get().multiprocess {
                    return Err(serde::de::Error::custom(
                        "InProcess callback found in multiprocess mode",
                    ));
                }
                let addr = variant_data.newtype_variant::<usize>()?;
                let ptr = addr as *mut _;
                // SAFETY: We know we are in the same address space as the sender, so we can safely
                // reconstruct the Arc, that we previously leaked with `into_raw` during
                // serialization.
                // Attention: Code reviewers should carefully compare the deserialization here
                // with the serialization above.
                #[expect(unsafe_code)]
                let callback = unsafe { Box::from_raw(ptr) };
                Ok(CallbackSetter(CallbackSetterVariants::InProcess(*callback)))
            },
        }
    }
}

impl<'a, T> Deserialize<'a> for CallbackSetter<T>
where
    T: Serialize + Deserialize<'a> + Send + 'static,
{
    fn deserialize<D>(d: D) -> Result<CallbackSetter<T>, D::Error>
    where
        D: Deserializer<'a>,
    {
        d.deserialize_enum(
            "GenericCallback",
            &["CrossProcess", "InProcess"],
            LazyCallbackSetterVisitor {
                marker: PhantomData,
            },
        )
    }
}

enum CallbackSetterVariants<T>
where
    T: Serialize + Send + 'static,
{
    InProcess(crossbeam_channel::Sender<GenericCallback<T>>),
    Ipc(IpcReceiver<T>),
}

impl<T> CallbackSetter<T>
where
    T: Serialize + for<'de> Deserialize<'de> + Send + 'static,
{
    /// This sets the callback.
    pub fn set_callback<F: FnMut(Result<T, ipc_channel::Error>) + Send + 'static>(
        self,
        callback: F,
    ) {
        match self.0 {
            CallbackSetterVariants::InProcess(sender) => {
                let callback = GenericCallback::new(callback).expect("Could not create callback");
                sender.send(callback).expect("Could not send callback");
            },
            CallbackSetterVariants::Ipc(ipc_receiver) => {
                ROUTER.add_typed_route(ipc_receiver, Box::new(callback));
            },
        }
    }
}

/// This function should never be exported.
fn lazy_callback_inprocess<T>() -> (LazyCallback<T>, CallbackSetter<T>)
where
    T: Serialize + for<'de> Deserialize<'de> + Send + 'static,
{
    let (callback_sender, callback_receiver) = crossbeam_channel::bounded(1);
    let lazycallback = LazyCallback(LazyCallbackVariants::InProcess {
        callback_receiver: RefCell::new(Some(callback_receiver)),
        callback: OnceCell::new(),
    });

    let callback_setter = CallbackSetter(CallbackSetterVariants::InProcess(callback_sender));

    (lazycallback, callback_setter)
}

/// This function should never be exported.
fn lazy_callback_ipc<T>() -> (LazyCallback<T>, CallbackSetter<T>)
where
    T: Serialize + for<'de> Deserialize<'de> + Send + 'static,
{
    let (sender, receiver) = ipc_channel::ipc::channel().expect("Could not create channel");
    let callback = LazyCallback(LazyCallbackVariants::Ipc(sender));
    let callback_setter = CallbackSetter(CallbackSetterVariants::Ipc(receiver));
    (callback, callback_setter)
}

/// A LazyCallback is a Callback that will be initialized at a later date.
/// We return the 'LazyCallback' which is a GenericCallback.
/// We also return a 'CallbackSetter' where the callback can be set at a later date.
pub fn lazy_callback<T>() -> (LazyCallback<T>, CallbackSetter<T>)
where
    T: Serialize + for<'de> Deserialize<'de> + Send + 'static,
{
    if opts::get().multiprocess || opts::get().force_ipc {
        lazy_callback_ipc()
    } else {
        lazy_callback_inprocess()
    }
}

#[cfg(test)]
mod single_process_callback_test {
    use crate::generic_channel::lazy_callback::{lazy_callback_inprocess, lazy_callback_ipc};

    #[test]
    fn lazy_callback_simple_inprocess() {
        let (callback, callback_setter) = lazy_callback_inprocess();

        let t1 = std::thread::spawn(move || {
            callback.send(true).expect("Could not send");
        });

        let (sender, receiver) = crossbeam_channel::bounded(1);
        let t2 = std::thread::spawn(move || {
            std::thread::sleep(std::time::Duration::from_secs(1));
            callback_setter.set_callback(move |value| {
                sender.send(value).expect("Could not send");
            });
        });

        t1.join().expect("error joining thread");
        t2.join().expect("error joining thread");
        assert_eq!(receiver.recv().unwrap().unwrap(), true);
    }

    #[test]
    fn lazy_callback_simple_ipc() {
        let (callback, callback_setter) = lazy_callback_ipc();

        let t1 = std::thread::spawn(move || {
            callback.send(true).expect("Could not send");
        });

        let (sender, receiver) = crossbeam_channel::bounded(1);
        let t2 = std::thread::spawn(move || {
            std::thread::sleep(std::time::Duration::from_secs(1));
            callback_setter.set_callback(move |value| {
                sender.send(value).expect("Could not send");
            });
        });

        t1.join().expect("error joining thread");
        t2.join().expect("error joining thread");
        assert_eq!(receiver.recv().unwrap().unwrap(), true);
    }
}
