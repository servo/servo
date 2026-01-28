/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

//! # Generic Callbacks
//!
//! When sending cross-process messages, we sometimes want to run custom callbacks when the
//! recipient has finished processing. The callback should run in the sender's address space, and
//! could be something like enqueuing a task.
//! In Multi-process mode we can implement this by providing an `IpcSender` to the recipient,
//! which the recipient can use to send some data back to the senders process.
//! To avoid blocking the sender, we can pass the callback to the ROUTER, which runs the callback
//! when receiving the Ipc message.
//! The callback will be run on every reply message from the recipient. `IpcSender`s are also
//! `Clone`able, so the Router will sequentialise callbacks.
//!
//! ## Callback scenario visualization
//!
//! The following visualization showcases how Ipc and the router thread are currently used
//! to run callbacks asynchronously on the sender process. The recipient may keep the
//! ReplySender alive and send an arbitrary amount of messages / replies.
//!
//! ```none
//!               Process A                      |              Process B
//!                                              |
//! +---------+   IPC: SendMessage(ReplySender)  |          +-------------+  clone  +-------------+
//! | Sender  |-------------------------------------------> |  Recipient  | ------> | ReplySender |
//! +---------+                                  |          +-------------+         +-------------+
//!   |                                          |                 |                       |
//!   | RegisterCallback A  +---------+          |  Send Reply 1   |        Send Reply 2   |
//!   + ------------------> | Router  | <--------------------------+-----------------------+
//!                         +---------+          |
//!                             | A(reply1)      |
//!                             | A(reply2)      |
//!                             |     ...        |
//!                             v                |
//!                                              |
//! ```
//!
//!
//! ## Optimizing single-process mode.
//!
//! In Single-process mode, there is no need for the Recipient to send an IpcReply,
//! since they are in the same address space and could just execute the callback directly.
//! Since we want to create an abstraction over such callbacks, we need to consider constraints
//! that the existing multiprocess Ipc solution imposes on us:
//!
//! - Support for `FnMut` callbacks (internal mutable state + multiple calls)
//! - The abstraction should be `Clone`able
//!
//! These constraints motivate the [GenericCallback] type, which supports `FnMut` callbacks
//! and is clonable. This requires wrapping the callback with `Arc<Mutex<>>`, which also adds
//! synchronization, which could be something that existing callbacks rely on.
//!
//! ### Future work
//!
//! - Further abstractions for callbacks with fewer constraints, e.g. callbacks
//!   which don't need to be cloned by the recipient, or non-mutable callbacks.
//! - A tracing option to measure callback runtime and identify callbacks which misbehave (block)
//!   for a long time.

use std::fmt;
use std::marker::PhantomData;
use std::sync::{Arc, Mutex};

use ipc_channel::ipc::IpcSender;
use ipc_channel::router::ROUTER;
use malloc_size_of::{MallocSizeOf, MallocSizeOfOps};
use serde::de::VariantAccess;
use serde::{Deserialize, Deserializer, Serialize, Serializer};
use servo_config::opts;

use crate::generic_channel::{GenericReceiver, GenericReceiverVariants, SendError, SendResult};

/// The callback type of our messages.
///
/// This is equivalent to [TypedRouterHandler][ipc_channel::router::TypedRouterHandler],
/// except that this type is not wrapped in a Box.
/// The callback will be wrapped in either a Box or an Arc, depending on if it is run on
/// the router, or passed to the recipient.
pub type MsgCallback<T> = dyn FnMut(Result<T, ipc_channel::IpcError>) + Send;

/// A mechanism to run a callback in the process this callback was constructed in.
///
/// The GenericCallback can be sent cross-process (in multi-process mode). In this case
/// the callback will be executed on the [ROUTER] thread.
/// In single-process mode the callback will be executed directly.
pub struct GenericCallback<T>(GenericCallbackVariants<T>)
where
    T: Serialize + Send + 'static;

enum GenericCallbackVariants<T>
where
    T: Serialize + Send + 'static,
{
    CrossProcess(IpcSender<T>),
    InProcess(Arc<Mutex<MsgCallback<T>>>),
}

impl<T> Clone for GenericCallback<T>
where
    T: Serialize + Send + 'static,
{
    fn clone(&self) -> Self {
        let variant = match &self.0 {
            GenericCallbackVariants::CrossProcess(sender) => {
                GenericCallbackVariants::CrossProcess((*sender).clone())
            },
            GenericCallbackVariants::InProcess(callback) => {
                GenericCallbackVariants::InProcess(callback.clone())
            },
        };
        GenericCallback(variant)
    }
}

impl<T> MallocSizeOf for GenericCallback<T>
where
    T: Serialize + Send + 'static,
{
    fn size_of(&self, _ops: &mut MallocSizeOfOps) -> usize {
        0
    }
}

impl<T> GenericCallback<T>
where
    T: for<'de> Deserialize<'de> + Serialize + Send + 'static,
{
    /// Creates a new GenericCallback.
    ///
    /// The callback should not do any heavy work and not block.
    pub fn new<F: FnMut(Result<T, ipc_channel::IpcError>) + Send + 'static>(
        mut callback: F,
    ) -> Result<Self, ipc_channel::IpcError> {
        let generic_callback = if opts::get().multiprocess || opts::get().force_ipc {
            let (ipc_sender, ipc_receiver) = ipc_channel::ipc::channel()?;
            let new_callback = move |msg: Result<T, ipc_channel::SerDeError>| {
                callback(msg.map_err(|error| error.into()))
            };
            ROUTER.add_typed_route(ipc_receiver, Box::new(new_callback));
            GenericCallback(GenericCallbackVariants::CrossProcess(ipc_sender))
        } else {
            let callback = Arc::new(Mutex::new(callback));
            GenericCallback(GenericCallbackVariants::InProcess(callback))
        };
        Ok(generic_callback)
    }

    /// Produces a GenericCallback and a channel. You can block on this channel for the result.
    pub fn new_blocking() -> Result<(Self, GenericReceiver<T>), ipc_channel::IpcError> {
        if opts::get().multiprocess || opts::get().force_ipc {
            let (sender, receiver) = ipc_channel::ipc::channel()?;
            let generic_callback = GenericCallback(GenericCallbackVariants::CrossProcess(sender));
            let receiver = GenericReceiver(GenericReceiverVariants::Ipc(receiver));
            Ok((generic_callback, receiver))
        } else {
            let (sender, receiver) = crossbeam_channel::bounded(1);
            let callback = Arc::new(Mutex::new(move |msg| {
                if sender.send(msg).is_err() {
                    log::error!("Error in callback");
                }
            }));
            let generic_callback = GenericCallback(GenericCallbackVariants::InProcess(callback));
            let receiver = GenericReceiver(GenericReceiverVariants::Crossbeam(receiver));
            Ok((generic_callback, receiver))
        }
    }

    /// Send `value` to the callback.
    ///
    /// Note that a return value of `Ok()` simply means that value was sent successfully
    /// to the callback. The callback itself does not return any value.
    /// The caller may not assume that the callback is executed synchronously.
    pub fn send(&self, value: T) -> SendResult {
        match &self.0 {
            GenericCallbackVariants::CrossProcess(sender) => {
                sender.send(value).map_err(|error| match error {
                    ipc_channel::IpcError::SerializationError(ser_de_error) => {
                        SendError::SerializationError(ser_de_error.to_string())
                    },
                    ipc_channel::IpcError::Io(_) | ipc_channel::IpcError::Disconnected => {
                        SendError::Disconnected
                    },
                })
            },
            GenericCallbackVariants::InProcess(callback) => {
                let mut cb = callback.lock().expect("poisoned");
                (*cb)(Ok(value));
                Ok(())
            },
        }
    }
}

impl<T> Serialize for GenericCallback<T>
where
    T: Serialize + Send + 'static,
{
    fn serialize<S: Serializer>(&self, s: S) -> Result<S::Ok, S::Error> {
        match &self.0 {
            GenericCallbackVariants::CrossProcess(sender) => {
                s.serialize_newtype_variant("GenericCallback", 0, "CrossProcess", sender)
            },
            // The only reason we need / want serialization in single-process mode is to support
            // sending GenericCallbacks over existing IPC channels. This allows us to
            // incrementally port IPC channels to the GenericChannel, without needing to follow a
            // top-to-bottom approach.
            // Long-term we can remove this branch in the code again and replace it with
            // unreachable, since likely all IPC channels would be GenericChannels.
            GenericCallbackVariants::InProcess(wrapped_callback) => {
                if opts::get().multiprocess {
                    return Err(serde::ser::Error::custom(
                        "InProcess callback can't be serialized in multiprocess mode",
                    ));
                }
                // Due to the signature of `serialize` we need to clone the Arc to get an owned
                // pointer we can leak.
                // We additionally need to Box to get a thin pointer.
                let cloned_callback = Box::new(wrapped_callback.clone());
                let sender_clone_addr = Box::leak(cloned_callback) as *mut Arc<_> as usize;
                s.serialize_newtype_variant("GenericCallback", 1, "InProcess", &sender_clone_addr)
            },
        }
    }
}

struct GenericCallbackVisitor<T> {
    marker: PhantomData<T>,
}

impl<'de, T> serde::de::Visitor<'de> for GenericCallbackVisitor<T>
where
    T: Serialize + Deserialize<'de> + Send + 'static,
{
    type Value = GenericCallback<T>;

    fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        formatter.write_str("a GenericCallback variant")
    }

    fn visit_enum<A>(self, data: A) -> Result<Self::Value, A::Error>
    where
        A: serde::de::EnumAccess<'de>,
    {
        #[derive(Deserialize)]
        enum GenericCallbackVariantNames {
            CrossProcess,
            InProcess,
        }

        let (variant_name, variant_data): (GenericCallbackVariantNames, _) = data.variant()?;

        match variant_name {
            GenericCallbackVariantNames::CrossProcess => variant_data
                .newtype_variant::<IpcSender<T>>()
                .map(|sender| GenericCallback(GenericCallbackVariants::CrossProcess(sender))),
            GenericCallbackVariantNames::InProcess => {
                if opts::get().multiprocess {
                    return Err(serde::de::Error::custom(
                        "InProcess callback found in multiprocess mode",
                    ));
                }
                let addr = variant_data.newtype_variant::<usize>()?;
                let ptr = addr as *mut Arc<Mutex<_>>;
                // SAFETY: We know we are in the same address space as the sender, so we can safely
                // reconstruct the Arc, that we previously leaked with `into_raw` during
                // serialization.
                // Attention: Code reviewers should carefully compare the deserialization here
                // with the serialization above.
                #[expect(unsafe_code)]
                let callback = unsafe { Box::from_raw(ptr) };
                Ok(GenericCallback(GenericCallbackVariants::InProcess(
                    *callback,
                )))
            },
        }
    }
}

impl<'a, T> Deserialize<'a> for GenericCallback<T>
where
    T: Serialize + Deserialize<'a> + Send + 'static,
{
    fn deserialize<D>(d: D) -> Result<GenericCallback<T>, D::Error>
    where
        D: Deserializer<'a>,
    {
        d.deserialize_enum(
            "GenericCallback",
            &["CrossProcess", "InProcess"],
            GenericCallbackVisitor {
                marker: PhantomData,
            },
        )
    }
}

impl<T> fmt::Debug for GenericCallback<T>
where
    T: Serialize + Send + 'static,
{
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "GenericCallback(..)")
    }
}

#[cfg(test)]
mod single_process_callback_test {
    use std::sync::Arc;
    use std::sync::atomic::{AtomicUsize, Ordering};

    use crate::generic_channel::GenericCallback;

    #[test]
    fn generic_callback() {
        let number = Arc::new(AtomicUsize::new(0));
        let number_clone = number.clone();
        let callback = move |msg: Result<usize, ipc_channel::IpcError>| {
            number_clone.store(msg.unwrap(), Ordering::SeqCst)
        };
        let generic_callback = GenericCallback::new(callback).unwrap();
        std::thread::scope(|s| {
            s.spawn(move || generic_callback.send(42));
        });
        assert_eq!(number.load(Ordering::SeqCst), 42);
    }

    #[test]
    fn generic_callback_via_generic_sender() {
        let number = Arc::new(AtomicUsize::new(0));
        let number_clone = number.clone();
        let callback = move |msg: Result<usize, ipc_channel::IpcError>| {
            number_clone.store(msg.unwrap(), Ordering::SeqCst)
        };
        let generic_callback = GenericCallback::new(callback).unwrap();
        let (tx, rx) = crate::generic_channel::channel().unwrap();

        tx.send(generic_callback).unwrap();
        std::thread::scope(|s| {
            s.spawn(move || {
                let callback = rx.recv().unwrap();
                callback.send(42).unwrap();
            });
        });
        assert_eq!(number.load(Ordering::SeqCst), 42);
    }

    #[test]
    fn generic_callback_via_ipc_sender() {
        let number = Arc::new(AtomicUsize::new(0));
        let number_clone = number.clone();
        let callback = move |msg: Result<usize, ipc_channel::IpcError>| {
            number_clone.store(msg.unwrap(), Ordering::SeqCst)
        };
        let generic_callback = GenericCallback::new(callback).unwrap();
        let (tx, rx) = ipc_channel::ipc::channel().unwrap();

        tx.send(generic_callback).unwrap();
        std::thread::scope(|s| {
            s.spawn(move || {
                let callback = rx.recv().unwrap();
                callback.send(42).unwrap();
            });
        });
        assert_eq!(number.load(Ordering::SeqCst), 42);
    }

    #[test]
    fn generic_callback_blocking() {
        let (callback, receiver) = GenericCallback::new_blocking().unwrap();
        std::thread::spawn(move || {
            std::thread::sleep(std::time::Duration::from_secs(1));
            assert!(callback.send(42).is_ok());
        });
        assert_eq!(receiver.recv().unwrap(), 42);
    }
}
