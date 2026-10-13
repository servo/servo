/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use js::context::JSContext;
use script_bindings::reflector::DomObject;
pub(crate) use script_bindings::routed_promise::RoutedPromiseListener;
use serde::Serialize;
use serde::de::DeserializeOwned;
use servo_base::generic_channel::GenericCallback;

use crate::dom::bindings::refcounted::{Trusted, TrustedPromise};
use crate::dom::promise::RootedPromise;
use crate::tasks::task_source::TaskSource;

pub(crate) struct RoutedPromiseContext<
    R: Serialize + DeserializeOwned + Send,
    T: RoutedPromiseListener<crate::DomTypeHolder, R> + DomObject,
> {
    trusted: TrustedPromise,
    receiver: Trusted<T>,
    _phantom: std::marker::PhantomData<R>,
}

impl<
    R: Serialize + DeserializeOwned + Send,
    T: RoutedPromiseListener<crate::DomTypeHolder, R> + DomObject,
> RoutedPromiseContext<R, T>
{
    fn response(self, cx: &mut JSContext, response: R) {
        let promise = self.trusted.root(cx);
        self.receiver.root().handle_response(cx, response, &promise);
    }
}

pub(crate) fn callback_promise<
    R: Serialize + DeserializeOwned + Send + 'static,
    T: RoutedPromiseListener<crate::DomTypeHolder, R> + DomObject + 'static,
>(
    promise: &RootedPromise,
    receiver: &T,
    task_source: TaskSource,
) -> GenericCallback<R> {
    let task_source = task_source.to_sendable();
    let mut trusted: Option<TrustedPromise> = Some(TrustedPromise::from(promise));
    let trusted_receiver = Trusted::new(receiver);
    GenericCallback::new(move |message| {
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
        task_source.queue(task!(routed_promise_task: move|cx| {
            context.response(cx, message.unwrap());
        }));
    })
    .expect("Could not create callback in script.")
}
