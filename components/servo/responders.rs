/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

/// Sender for errors raised by delegate request objects.
///
/// This allows errors to be raised asynchronously.
pub(crate) trait DelegateErrorSender {
    fn raise_response_send_error(&self, error: bincode::Error);
}
