/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

// https://w3c.github.io/ServiceWorker/#windowclient

[Pref="dom_serviceworker_enabled", Exposed=ServiceWorker]
interface WindowClient : Client {
  // readonly attribute VisibilityState visibilityState;
  // readonly attribute boolean focused;
  // [SameObject] readonly attribute FrozenArray<USVString> any ancestorOrigins;
  [NewObject] Promise<WindowClient> focus();
  [NewObject] Promise<WindowClient?> navigate(USVString url);
};
