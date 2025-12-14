/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

//! Enum wrappers to be able to select different channel implementations at runtime.

use std::fmt;
use std::fmt::Display;
use std::marker::PhantomData;

use ipc_channel::ipc::IpcError;
use ipc_channel::router::ROUTER;
use malloc_size_of::{MallocSizeOf, MallocSizeOfOps};
use serde::de::VariantAccess;
use serde::{Deserialize, Deserializer, Serialize, Serializer};
use servo_config::opts;

mod callback;
pub use callback::GenericCallback;
mod oneshot;
pub use oneshot::{GenericOneshotReceiver, GenericOneshotSender, oneshot};

/// Abstraction of the ability to send a particular type of message cross-process.
/// This can be used to ease the use of GenericSender sub-fields.
pub trait GenericSend<T>
where
    T: serde::Serialize + for<'de> serde::Deserialize<'de>,
{
    /// send message T
    fn send(&self, _: T) -> SendResult;
    /// get underlying sender
    fn sender(&self) -> GenericSender<T>;
}

/// A GenericSender that sends messages to a [GenericReceiver].
///
/// The sender supports sending messages cross-process, if servo is run in multiprocess mode.
pub struct GenericSender<T: Serialize>(GenericSenderVariants<T>);

/// The actual GenericSender variant.
///
/// This enum is private, so that outside code can't construct a GenericSender itself.
/// This ensures that users can't construct a crossbeam variant in multiprocess mode.
enum GenericSenderVariants<T: Serialize> {
    Ipc(ipc_channel::ipc::IpcSender<T>),
    /// A crossbeam-channel. To keep the API in sync with the Ipc variant when using a Router,
    /// which propagates the IPC error, the inner type is a Result.
    /// In the IPC case, the Router deserializes the message, which can fail, and sends
    /// the result to a crossbeam receiver.
    /// The crossbeam channel does not involve serializing, so we can't have this error,
    /// but replicating the API allows us to have one channel type as the receiver
    /// after routing the receiver .
    Crossbeam(crossbeam_channel::Sender<Result<T, ipc_channel::Error>>),
}

fn serialize_generic_sender_variants<T: Serialize, S: Serializer>(
    value: &GenericSenderVariants<T>,
    s: S,
) -> Result<S::Ok, S::Error> {
    match value {
        GenericSenderVariants::Ipc(sender) => {
            s.serialize_newtype_variant("GenericSender", 0, "Ipc", sender)
        },
        // All GenericSenders will be IPC channels in multi-process mode, so sending a
        // GenericChannel over existing IPC channels is no problem and won't fail.
        // In single-process mode, we can also send GenericSenders over other GenericSenders
        // just fine, since no serialization is required.
        // The only reason we need / want serialization is to support sending GenericSenders
        // over existing IPC channels **in single process mode**. This allows us to
        // incrementally port channels to the GenericChannel, without needing to follow a
        // top-to-bottom approach.
        // Long-term we can remove this branch in the code again and replace it with
        // unreachable, since likely all IPC channels would be GenericChannels.
        GenericSenderVariants::Crossbeam(sender) => {
            if opts::get().multiprocess {
                return Err(serde::ser::Error::custom(
                    "Crossbeam channel found in multiprocess mode!",
                ));
            } // We know everything is in one address-space, so we can "serialize" the sender by
            // sending a leaked Box pointer.
            let sender_clone_addr = Box::leak(Box::new(sender.clone())) as *mut _ as usize;
            s.serialize_newtype_variant("GenericSender", 1, "Crossbeam", &sender_clone_addr)
        },
    }
}

impl<T: Serialize> Serialize for GenericSender<T> {
    fn serialize<S: Serializer>(&self, s: S) -> Result<S::Ok, S::Error> {
        serialize_generic_sender_variants(&self.0, s)
    }
}

struct GenericSenderVisitor<T> {
    marker: PhantomData<T>,
}

impl<'de, T: Serialize + Deserialize<'de>> serde::de::Visitor<'de> for GenericSenderVisitor<T> {
    type Value = GenericSenderVariants<T>;

    fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        formatter.write_str("a GenericSender variant")
    }

    fn visit_enum<A>(self, data: A) -> Result<Self::Value, A::Error>
    where
        A: serde::de::EnumAccess<'de>,
    {
        #[derive(Deserialize)]
        enum GenericSenderVariantNames {
            Ipc,
            Crossbeam,
        }

        let (variant_name, variant_data): (GenericSenderVariantNames, _) = data.variant()?;

        match variant_name {
            GenericSenderVariantNames::Ipc => variant_data
                .newtype_variant::<ipc_channel::ipc::IpcSender<T>>()
                .map(|sender| GenericSenderVariants::Ipc(sender)),
            GenericSenderVariantNames::Crossbeam => {
                if opts::get().multiprocess {
                    return Err(serde::de::Error::custom(
                        "Crossbeam channel found in multiprocess mode!",
                    ));
                }
                let addr = variant_data.newtype_variant::<usize>()?;
                let ptr = addr as *mut crossbeam_channel::Sender<Result<T, ipc_channel::Error>>;
                // SAFETY: We know we are in the same address space as the sender, so we can safely
                // reconstruct the Box.
                #[expect(unsafe_code)]
                let sender = unsafe { Box::from_raw(ptr) };
                Ok(GenericSenderVariants::Crossbeam(*sender))
            },
        }
    }
}

impl<'a, T: Serialize + Deserialize<'a>> Deserialize<'a> for GenericSender<T> {
    fn deserialize<D>(d: D) -> Result<GenericSender<T>, D::Error>
    where
        D: Deserializer<'a>,
    {
        d.deserialize_enum(
            "GenericSender",
            &["Ipc", "Crossbeam"],
            GenericSenderVisitor {
                marker: PhantomData,
            },
        )
        .map(|variant| GenericSender(variant))
    }
}

impl<T> Clone for GenericSender<T>
where
    T: Serialize,
{
    fn clone(&self) -> Self {
        match self.0 {
            GenericSenderVariants::Ipc(ref chan) => {
                GenericSender(GenericSenderVariants::Ipc(chan.clone()))
            },
            GenericSenderVariants::Crossbeam(ref chan) => {
                GenericSender(GenericSenderVariants::Crossbeam(chan.clone()))
            },
        }
    }
}

impl<T: Serialize> fmt::Debug for GenericSender<T> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "Sender(..)")
    }
}

impl<T: Serialize> GenericSender<T> {
    #[inline]
    pub fn send(&self, msg: T) -> SendResult {
        match self.0 {
            GenericSenderVariants::Ipc(ref sender) => sender
                .send(msg)
                .map_err(|e| SendError::SerializationError(format!("{e}"))),
            GenericSenderVariants::Crossbeam(ref sender) => {
                sender.send(Ok(msg)).map_err(|_| SendError::Disconnected)
            },
        }
    }
}

impl<T: Serialize> MallocSizeOf for GenericSender<T> {
    fn size_of(&self, _ops: &mut MallocSizeOfOps) -> usize {
        0
    }
}

#[derive(Debug)]
pub enum SendError {
    Disconnected,
    SerializationError(String),
}

impl Display for SendError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{self:?}")
    }
}

pub type SendResult = Result<(), SendError>;

#[derive(Debug)]
pub enum ReceiveError {
    DeserializationFailed(String),
    /// Io Error. May occur when using IPC.
    Io(std::io::Error),
    /// The channel was closed.
    Disconnected,
}

impl From<IpcError> for ReceiveError {
    fn from(e: IpcError) -> Self {
        match e {
            IpcError::Disconnected => ReceiveError::Disconnected,
            IpcError::Bincode(reason) => ReceiveError::DeserializationFailed(reason.to_string()),
            IpcError::Io(reason) => ReceiveError::Io(reason),
        }
    }
}

impl From<crossbeam_channel::RecvError> for ReceiveError {
    fn from(_: crossbeam_channel::RecvError) -> Self {
        ReceiveError::Disconnected
    }
}

impl fmt::Display for ReceiveError {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            ReceiveError::DeserializationFailed(ref error) => {
                write!(fmt, "deserialization error: {error}")
            },
            ReceiveError::Io(ref error) => write!(fmt, "io error: {error}"),
            ReceiveError::Disconnected => write!(fmt, "disconnected"),
        }
    }
}

pub enum TryReceiveError {
    Empty,
    ReceiveError(ReceiveError),
}

impl From<ipc_channel::ipc::TryRecvError> for TryReceiveError {
    fn from(e: ipc_channel::ipc::TryRecvError) -> Self {
        match e {
            ipc_channel::ipc::TryRecvError::Empty => TryReceiveError::Empty,
            ipc_channel::ipc::TryRecvError::IpcError(inner) => {
                TryReceiveError::ReceiveError(inner.into())
            },
        }
    }
}

impl From<crossbeam_channel::TryRecvError> for TryReceiveError {
    fn from(e: crossbeam_channel::TryRecvError) -> Self {
        match e {
            crossbeam_channel::TryRecvError::Empty => TryReceiveError::Empty,
            crossbeam_channel::TryRecvError::Disconnected => {
                TryReceiveError::ReceiveError(ReceiveError::Disconnected)
            },
        }
    }
}

pub type RoutedReceiver<T> = crossbeam_channel::Receiver<Result<T, ipc_channel::Error>>;
pub type ReceiveResult<T> = Result<T, ReceiveError>;
pub type TryReceiveResult<T> = Result<T, TryReceiveError>;
pub type RoutedReceiverReceiveResult<T> =
    Result<Result<T, ipc_channel::Error>, crossbeam_channel::RecvError>;

pub fn to_receive_result<T>(receive_result: RoutedReceiverReceiveResult<T>) -> ReceiveResult<T> {
    match receive_result {
        Ok(Ok(msg)) => Ok(msg),
        Err(_crossbeam_recv_err) => Err(ReceiveError::Disconnected),
        Ok(Err(ipc_err)) => Err(ReceiveError::DeserializationFailed(ipc_err.to_string())),
    }
}

pub struct GenericReceiver<T>(GenericReceiverVariants<T>)
where
    T: for<'de> Deserialize<'de> + Serialize;

enum GenericReceiverVariants<T>
where
    T: for<'de> Deserialize<'de> + Serialize,
{
    Ipc(ipc_channel::ipc::IpcReceiver<T>),
    Crossbeam(RoutedReceiver<T>),
}

impl<T> GenericReceiver<T>
where
    T: for<'de> Deserialize<'de> + Serialize,
{
    #[inline]
    pub fn recv(&self) -> ReceiveResult<T> {
        match self.0 {
            GenericReceiverVariants::Ipc(ref receiver) => Ok(receiver.recv()?),
            GenericReceiverVariants::Crossbeam(ref receiver) => {
                // `recv()` returns an error if the channel is disconnected
                let msg = receiver.recv()?;
                // `msg` must be `ok` because the corresponding [`GenericSender::Crossbeam`] will
                // unconditionally send an `Ok(T)`
                Ok(msg.expect("Infallible"))
            },
        }
    }

    #[inline]
    pub fn try_recv(&self) -> TryReceiveResult<T> {
        match self.0 {
            GenericReceiverVariants::Ipc(ref receiver) => Ok(receiver.try_recv()?),
            GenericReceiverVariants::Crossbeam(ref receiver) => {
                let msg = receiver.try_recv()?;
                Ok(msg.expect("Infallible"))
            },
        }
    }

    /// Route to a crossbeam receiver, preserving any errors.
    ///
    /// For `Crossbeam` receivers this is a no-op, while for `Ipc` receivers
    /// this creates a route.
    #[inline]
    pub fn route_preserving_errors(self) -> RoutedReceiver<T>
    where
        T: Send + 'static,
    {
        match self.0 {
            GenericReceiverVariants::Ipc(ipc_receiver) => {
                let (crossbeam_sender, crossbeam_receiver) = crossbeam_channel::unbounded();
                let crossbeam_sender_clone = crossbeam_sender.clone();
                ROUTER.add_typed_route(
                    ipc_receiver,
                    Box::new(move |message| {
                        let _ = crossbeam_sender_clone.send(message);
                    }),
                );
                crossbeam_receiver
            },
            GenericReceiverVariants::Crossbeam(receiver) => receiver,
        }
    }
}

impl<T> Serialize for GenericReceiver<T>
where
    T: for<'de> Deserialize<'de> + Serialize,
{
    fn serialize<S: Serializer>(&self, s: S) -> Result<S::Ok, S::Error> {
        match &self.0 {
            GenericReceiverVariants::Ipc(receiver) => {
                s.serialize_newtype_variant("GenericReceiver", 0, "Ipc", receiver)
            },
            GenericReceiverVariants::Crossbeam(receiver) => {
                if opts::get().multiprocess {
                    return Err(serde::ser::Error::custom(
                        "Crossbeam channel found in multiprocess mode!",
                    ));
                } // We know everything is in one address-space, so we can "serialize" the receiver by
                // sending a leaked Box pointer.
                let receiver_clone_addr = Box::leak(Box::new(receiver.clone())) as *mut _ as usize;
                s.serialize_newtype_variant("GenericReceiver", 1, "Crossbeam", &receiver_clone_addr)
            },
        }
    }
}

struct GenericReceiverVisitor<T> {
    marker: PhantomData<T>,
}
impl<'de, T> serde::de::Visitor<'de> for GenericReceiverVisitor<T>
where
    T: for<'a> Deserialize<'a> + Serialize,
{
    type Value = GenericReceiver<T>;

    fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        formatter.write_str("a GenericReceiver variant")
    }

    fn visit_enum<A>(self, data: A) -> Result<Self::Value, A::Error>
    where
        A: serde::de::EnumAccess<'de>,
    {
        #[derive(Deserialize)]
        enum GenericReceiverVariantNames {
            Ipc,
            Crossbeam,
        }

        let (variant_name, variant_data): (GenericReceiverVariantNames, _) = data.variant()?;

        match variant_name {
            GenericReceiverVariantNames::Ipc => variant_data
                .newtype_variant::<ipc_channel::ipc::IpcReceiver<T>>()
                .map(|receiver| GenericReceiver(GenericReceiverVariants::Ipc(receiver))),
            GenericReceiverVariantNames::Crossbeam => {
                if opts::get().multiprocess {
                    return Err(serde::de::Error::custom(
                        "Crossbeam channel found in multiprocess mode!",
                    ));
                }
                let addr = variant_data.newtype_variant::<usize>()?;
                let ptr = addr as *mut RoutedReceiver<T>;
                // SAFETY: We know we are in the same address space as the sender, so we can safely
                // reconstruct the Box.
                #[expect(unsafe_code)]
                let receiver = unsafe { Box::from_raw(ptr) };
                Ok(GenericReceiver(GenericReceiverVariants::Crossbeam(
                    *receiver,
                )))
            },
        }
    }
}

impl<'a, T> Deserialize<'a> for GenericReceiver<T>
where
    T: for<'de> Deserialize<'de> + Serialize,
{
    fn deserialize<D>(d: D) -> Result<GenericReceiver<T>, D::Error>
    where
        D: Deserializer<'a>,
    {
        d.deserialize_enum(
            "GenericReceiver",
            &["Ipc", "Crossbeam"],
            GenericReceiverVisitor {
                marker: PhantomData,
            },
        )
    }
}

/// Private helper function to create a crossbeam based channel.
///
/// Do NOT make this function public!
fn new_generic_channel_crossbeam<T>() -> (GenericSender<T>, GenericReceiver<T>)
where
    T: Serialize + for<'de> serde::Deserialize<'de>,
{
    let (tx, rx) = crossbeam_channel::unbounded();
    (
        GenericSender(GenericSenderVariants::Crossbeam(tx)),
        GenericReceiver(GenericReceiverVariants::Crossbeam(rx)),
    )
}

fn new_generic_channel_ipc<T>() -> Result<(GenericSender<T>, GenericReceiver<T>), std::io::Error>
where
    T: Serialize + for<'de> serde::Deserialize<'de>,
{
    ipc_channel::ipc::channel().map(|(tx, rx)| {
        (
            GenericSender(GenericSenderVariants::Ipc(tx)),
            GenericReceiver(GenericReceiverVariants::Ipc(rx)),
        )
    })
}

/// Creates a Servo channel that can select different channel implementations based on multiprocess
/// mode or not. If the scenario doesn't require message to pass process boundary, a simple
/// crossbeam channel is preferred.
pub fn channel<T>() -> Option<(GenericSender<T>, GenericReceiver<T>)>
where
    T: for<'de> Deserialize<'de> + Serialize,
{
    if servo_config::opts::get().multiprocess || servo_config::opts::get().force_ipc {
        new_generic_channel_ipc().ok()
    } else {
        Some(new_generic_channel_crossbeam())
    }
}

#[cfg(test)]
mod single_process_channel_tests {
    //! These unit-tests test that ipc_channel and crossbeam_channel Senders and Receivers
    //! can be sent over each other without problems in single-process mode.
    //! In multiprocess mode we exclusively use `ipc_channel` anyway, which is ensured due
    //! to `channel()` being the only way to construct `GenericSender` and Receiver pairs.
    use crate::generic_channel::{new_generic_channel_crossbeam, new_generic_channel_ipc};

    #[test]
    fn generic_crossbeam_can_send() {
        let (tx, rx) = new_generic_channel_crossbeam();
        tx.send(5).expect("Send failed");
        let val = rx.recv().expect("Receive failed");
        assert_eq!(val, 5);
    }

    #[test]
    fn generic_crossbeam_ping_pong() {
        let (tx, rx) = new_generic_channel_crossbeam();
        let (tx2, rx2) = new_generic_channel_crossbeam();

        tx.send(tx2).expect("Send failed");

        std::thread::scope(|s| {
            s.spawn(move || {
                let reply_sender = rx.recv().expect("Receive failed");
                reply_sender.send(42).expect("Sending reply failed");
            });
        });
        let res = rx2.recv().expect("Receive of reply failed");
        assert_eq!(res, 42);
    }

    #[test]
    fn generic_ipc_ping_pong() {
        let (tx, rx) = new_generic_channel_ipc().unwrap();
        let (tx2, rx2) = new_generic_channel_ipc().unwrap();

        tx.send(tx2).expect("Send failed");

        std::thread::scope(|s| {
            s.spawn(move || {
                let reply_sender = rx.recv().expect("Receive failed");
                reply_sender.send(42).expect("Sending reply failed");
            });
        });
        let res = rx2.recv().expect("Receive of reply failed");
        assert_eq!(res, 42);
    }

    #[test]
    fn send_crossbeam_sender_over_ipc_channel() {
        let (tx, rx) = new_generic_channel_ipc().unwrap();
        let (tx2, rx2) = new_generic_channel_crossbeam();

        tx.send(tx2).expect("Send failed");

        std::thread::scope(|s| {
            s.spawn(move || {
                let reply_sender = rx.recv().expect("Receive failed");
                reply_sender.send(42).expect("Sending reply failed");
            });
        });
        let res = rx2.recv().expect("Receive of reply failed");
        assert_eq!(res, 42);
    }

    #[test]
    fn send_generic_ipc_channel_over_crossbeam() {
        let (tx, rx) = new_generic_channel_crossbeam();
        let (tx2, rx2) = new_generic_channel_ipc().unwrap();

        tx.send(tx2).expect("Send failed");

        std::thread::scope(|s| {
            s.spawn(move || {
                let reply_sender = rx.recv().expect("Receive failed");
                reply_sender.send(42).expect("Sending reply failed");
            });
        });
        let res = rx2.recv().expect("Receive of reply failed");
        assert_eq!(res, 42);
    }

    #[test]
    fn send_crossbeam_receiver_over_ipc_channel() {
        let (tx, rx) = new_generic_channel_ipc().unwrap();
        let (tx2, rx2) = new_generic_channel_crossbeam();

        tx.send(rx2).expect("Send failed");
        tx2.send(42).expect("Send failed");

        std::thread::scope(|s| {
            s.spawn(move || {
                let another_receiver = rx.recv().expect("Receive failed");
                let res = another_receiver.recv().expect("Receive failed");
                assert_eq!(res, 42);
            });
        });
    }
}
