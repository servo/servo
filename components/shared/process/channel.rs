//! Enum wrappers to be able to select different channel implementations at runtime.

use std::fmt;

use ipc_channel::router::ROUTER;
use serde::{Deserialize, Serialize};
use servo_config::opts;

#[derive(Deserialize, Serialize)]
pub enum Sender<T: Serialize> {
    Ipc(ipc::Sender<T>),
    Mpmc(mpmc::Sender<T>),
}

impl<T> Clone for Sender<T>
where
    T: Serialize,
{
    fn clone(&self) -> Self {
        match *self {
            Sender::Ipc(ref chan) => Sender::Ipc(chan.clone()),
            Sender::Mpmc(ref chan) => Sender::Mpmc(chan.clone()),
        }
    }
}

impl<T: Serialize> fmt::Debug for Sender<T> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "Sender(..)")
    }
}

impl<T: Serialize> Sender<T> {
    #[inline]
    pub fn send(&self, msg: T) -> SendResult {
        match *self {
            Sender::Ipc(ref sender) => sender.send(msg).map_err(|_| SendError),
            Sender::Mpmc(ref sender) => sender.send(msg).map_err(|_| SendError),
        }
    }
}

#[derive(Debug)]
pub struct SendError;
pub type SendResult = Result<(), SendError>;

#[derive(Debug)]
pub struct ReceiveError;
pub type ReceiveResult<T> = Result<T, ReceiveError>;

pub enum Receiver<T>
where
    T: for<'de> Deserialize<'de> + Serialize,
{
    Ipc(ipc::Receiver<T>),
    Mpmc(mpmc::Receiver<T>),
}

impl<T> Receiver<T>
where
    T: for<'de> Deserialize<'de> + Serialize,
{
    pub fn recv(&self) -> ReceiveResult<T> {
        match *self {
            Receiver::Ipc(ref receiver) => receiver.recv().map_err(|_| ReceiveError),
            Receiver::Mpmc(ref receiver) => receiver.recv().map_err(|_| ReceiveError),
        }
    }

    pub fn try_recv(&self) -> ReceiveResult<T> {
        match *self {
            Receiver::Ipc(ref receiver) => receiver.try_recv().map_err(|_| ReceiveError),
            Receiver::Mpmc(ref receiver) => receiver.try_recv().map_err(|_| ReceiveError),
        }
    }

    pub fn into_inner(self) -> crossbeam_channel::Receiver<T>
    where
        T: Send + 'static,
    {
        match self {
            Receiver::Ipc(receiver) => {
                ROUTER.route_ipc_receiver_to_new_crossbeam_receiver(receiver)
            },
            Receiver::Mpmc(receiver) => receiver.into_inner(),
        }
    }
}

/// Creates a Servo channel that can select different channel implementations based on multiprocess
/// mode or not. If the scenario doesn't require message to pass process boundary, a simple
/// crossbeam channel is preferred.
pub fn channel<T>() -> Option<(Sender<T>, Receiver<T>)>
where
    T: for<'de> Deserialize<'de> + Serialize,
{
    if opts::multiprocess() {
        ipc::channel()
            .map(|(tx, rx)| (Sender::Ipc(tx), Receiver::Ipc(rx)))
            .ok()
    } else {
        mpmc::channel()
            .map(|(tx, rx)| (Sender::Mpmc(tx), Receiver::Mpmc(rx)))
            .ok()
    }
}

mod ipc {
    use std::io;

    use serde::{Deserialize, Serialize};

    pub type Sender<T> = ipc_channel::ipc::IpcSender<T>;
    pub type Receiver<T> = ipc_channel::ipc::IpcReceiver<T>;

    pub fn channel<T: Serialize + for<'de> Deserialize<'de>>(
    ) -> Result<(Sender<T>, Receiver<T>), io::Error> {
        ipc_channel::ipc::channel()
    }
}

mod mpmc {
    use serde::{Deserialize, Deserializer, Serialize, Serializer};

    macro_rules! unreachable_serializable {
        ($name:ident) => {
            impl<T> Serialize for $name<T> {
                fn serialize<S: Serializer>(&self, _: S) -> Result<S::Ok, S::Error> {
                    unreachable!();
                }
            }

            impl<'a, T> Deserialize<'a> for $name<T> {
                fn deserialize<D>(_: D) -> Result<$name<T>, D::Error>
                where
                    D: Deserializer<'a>,
                {
                    unreachable!();
                }
            }
        };
    }

    pub struct Sender<T>(crossbeam_channel::Sender<T>);
    pub struct Receiver<T>(crossbeam_channel::Receiver<T>);

    impl<T> Clone for Sender<T> {
        fn clone(&self) -> Self {
            Sender(self.0.clone())
        }
    }

    impl<T> Sender<T> {
        #[inline]
        pub fn send(&self, data: T) -> Result<(), crossbeam_channel::SendError<T>> {
            self.0.send(data)
        }
    }

    impl<T> Receiver<T> {
        #[inline]
        pub fn recv(&self) -> Result<T, crossbeam_channel::RecvError> {
            self.0.recv()
        }
        #[inline]
        pub fn try_recv(&self) -> Result<T, crossbeam_channel::TryRecvError> {
            self.0.try_recv()
        }
        pub fn into_inner(self) -> crossbeam_channel::Receiver<T> {
            self.0
        }
    }

    pub fn channel<T>() -> Result<(Sender<T>, Receiver<T>), ()> {
        let (sender, receiver) = crossbeam_channel::unbounded();
        Ok((Sender(sender), Receiver(receiver)))
    }

    unreachable_serializable!(Receiver);
    unreachable_serializable!(Sender);
}
