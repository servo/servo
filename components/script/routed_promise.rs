/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use std::rc::Rc;

use ipc_channel::ipc::{self, IpcSender};
use ipc_channel::router::ROUTER;
use serde::Serialize;
use serde::de::DeserializeOwned;

use crate::dom::bindings::refcounted::{Trusted, TrustedPromise};
use crate::dom::bindings::reflector::{DomGlobal, DomObject};
use crate::dom::promise::Promise;
use crate::script_runtime::CanGc;

pub(crate) trait RoutedPromiseListener {
    type Response: Serialize + DeserializeOwned + Send;

    fn handle_response(&self, response: Self::Response, promise: &Rc<Promise>, can_gc: CanGc);
}

pub(crate) struct RoutedPromiseContext<T: RoutedPromiseListener + DomObject> {
    trusted: TrustedPromise,
    receiver: Trusted<T>,
}

impl<T: RoutedPromiseListener + DomObject> RoutedPromiseContext<T> {
    #[cfg_attr(crown, allow(crown::unrooted_must_root))]
    fn response(self, response: T::Response, can_gc: CanGc) {
        let promise = self.trusted.root();
        self.receiver
            .root()
            .handle_response(response, &promise, can_gc);
    }
}

pub(crate) fn route_promise<T: RoutedPromiseListener + DomObject + 'static>(
    promise: &Rc<Promise>,
    receiver: &T,
) -> IpcSender<T::Response> {
    let (action_sender, action_receiver) = ipc::channel().unwrap();
    let task_source = receiver
        .global()
        .task_manager()
        .dom_manipulation_task_source()
        .to_sendable();
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
            };
            task_source.queue(task!(routed_promise_task: move|| {
                context.response(message.unwrap(), CanGc::note());
            }));
        }),
    );
    action_sender
}
