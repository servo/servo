/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

// https://slightlyoff.github.io/ServiceWorker/spec/service_worker/#client

// [Exposed=ServiceWorker]
[Pref="dom.serviceworker.enabled"]
interface Client {
  readonly attribute USVString url;
  readonly attribute FrameType frameType;
  readonly attribute DOMString id;
  //void postMessage(any message, optional sequence<Transferable> transfer);
};

enum FrameType {
  "auxiliary",
  "top-level",
  "nested",
  "none"
};
