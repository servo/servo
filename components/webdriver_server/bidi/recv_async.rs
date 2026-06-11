//! Bridge the gap between sync channels and async BiDi handler.
//!
//! ## FAQ
//!
//! ### Why do we need async?
//!
//! As of now, WebDriver BiDi already has over 60 commands, each with two or
//! three async steps on average. Actor model can lead to an unmaintainable
//! number of states.
//!
//! ### Why spawn_block for each recv?
//!
//! Yes. spawn_blocking is bad. There are at least two more ideal ways:
//!
//! 1. Support recv_async natively (blocked by crossbeam)
//! 2. Poll them on a single spawn_blocking thread (blocked by GenericReceiver)
//!
//! To keep the initial BiDi PR minimal, this is left for future work.

use serde::{Serialize, de::DeserializeOwned};
use servo_base::generic_channel::{GenericOneshotReceiver, ReceiveError};
use tokio::task::JoinError;

pub(crate) trait AsyncReceiver<T> {
    fn recv_async(&self) -> impl Future<Output = ReceiveAsyncResult<T>>;
}

pub(crate) trait AsyncOneshotReceiver<T> {
    fn recv_async(self) -> impl Future<Output = ReceiveAsyncResult<T>>;
}

pub(crate) enum ReceiveAsyncError {
    Disconnedted,
    ReceiveError(ReceiveError),
    TaskJoinError(JoinError),
}

pub(crate) type ReceiveAsyncResult<T> = Result<T, ReceiveAsyncError>;

impl<T: Send + 'static> AsyncReceiver<T> for crossbeam_channel::Receiver<T> {
    async fn recv_async(&self) -> ReceiveAsyncResult<T> {
        let rx = self.clone();
        tokio::task::spawn_blocking(move || rx.recv().map_err(|_| ReceiveAsyncError::Disconnedted))
            .await
            .map_err(ReceiveAsyncError::TaskJoinError)?
    }
}

impl<T> AsyncOneshotReceiver<T> for GenericOneshotReceiver<T>
where
    T: DeserializeOwned + Send + Serialize + 'static,
{
    async fn recv_async(self) -> ReceiveAsyncResult<T> {
        tokio::task::spawn_blocking(move || {
            self.recv().map_err(|_| ReceiveAsyncError::Disconnedted)
        })
        .await
        .map_err(ReceiveAsyncError::TaskJoinError)?
    }
}
