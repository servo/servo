use ipc_channel::IpcError;
use serde::{Serialize, de::DeserializeOwned};
use servo_base::generic_channel::GenericCallback;
use tokio::sync::oneshot::{self, Receiver};

pub(crate) fn new_oneshot_callback<T>() -> (GenericCallback<T>, Receiver<Result<T, IpcError>>)
where
    T: std::fmt::Debug + DeserializeOwned + Serialize + Send + 'static,
{
    let (sender, receiver) = oneshot::channel();
    let mut maybe_sender = Some(sender);

    let callback = GenericCallback::new(move |t| {
        if let Some(sender) = maybe_sender.take() {
            if let Err(e) = sender.send(t) {
                log::warn!("Sending callback channel failed: {e:?}");
            }
        }
    })
    .unwrap();

    (callback, receiver)
}
