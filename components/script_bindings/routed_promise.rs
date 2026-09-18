/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use js::context::JSContext;
use serde::Serialize;
use serde::de::DeserializeOwned;
use crate::DomTypes;
use crate::interfaces::PromiseHelpers;

pub trait RoutedPromiseListener<D: DomTypes, R: Serialize + DeserializeOwned + Send> {
    fn handle_response(
        &self,
        cx: &mut JSContext,
        response: R,
        promise: &<D::Promise as PromiseHelpers<D>>::StackRoot,
    );
}
