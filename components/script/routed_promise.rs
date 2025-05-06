/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use std::rc::Rc;

use ipc_channel::ipc::{self, IpcSender};
use ipc_channel::router::ROUTER;
use serde::Serialize;
use serde::de::DeserializeOwned;

use crate::dom::bindings::refcounted::{Trusted, TrustedPromise};
use crate::dom::bindings::reflector::DomObject;
use crate::dom::promise::Promise;
use crate::script_runtime::CanGc;
use crate::task_source::TaskSource;

pub(crate) trait RoutedPromiseListener<R: Serialize + DeserializeOwned + Send> {
    fn handle_response(&self, response: R, promise: &Rc<Promise>, can_gc: CanGc);
}

pub(crate) struct RoutedPromiseContext<
    R: Serialize + DeserializeOwned + Send,
    T: RoutedPromiseListener<R> + DomObject,
> {
    trusted: TrustedPromise,
    receiver: Trusted<T>,
    _phantom: std::marker::PhantomData<R>,
}

impl<R: Serialize + DeserializeOwned + Send, T: RoutedPromiseListener<R> + DomObject>
    RoutedPromiseContext<R, T>
{
    fn response(self, response: R, can_gc: CanGc) {
        let promise = self.trusted.root();
        self.receiver
            .root()
            .handle_response(response, &promise, can_gc);
    }
}

pub(crate) fn route_promise<
    R: Serialize + DeserializeOwned + Send + 'static,
    T: RoutedPromiseListener<R> + DomObject + 'static,
>(
    promise: &Rc<Promise>,
    receiver: &T,
    task_source: TaskSource,
) -> IpcSender<R> {
    let (action_sender, action_receiver) = ipc::channel().unwrap();
    let task_source = task_source.to_sendable();
    let mut trusted: Option<TrustedPromise> = Some(TrustedPromise::new(promise.clone()));
    let trusted_receiver = Trusted::new(receiver);
    ROUTER.add_typed_route(
        action_receiver,
        Box::new(move |message| {
            let trusted = if let Some(trusted) = trusted.take() {
                trusted
            } else {
                error!("RoutedPromiseListener callback called twice!");
                return;
            };

            let context = RoutedPromiseContext {
                trusted,
                receiver: trusted_receiver.clone(),
                _phantom: Default::default(),
            };
            task_source.queue(task!(routed_promise_task: move|| {
                context.response(message.unwrap(), CanGc::note());
            }));
        }),
    );
    action_sender
}
