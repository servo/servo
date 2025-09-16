/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

// https://dom.spec.whatwg.org/#abortsignal

[Exposed=*, Pref="dom_abort_controller_enabled"]
interface AbortSignal : EventTarget {
  [NewObject] static AbortSignal abort(optional any reason);
  // [Exposed=(Window,Worker), NewObject] static AbortSignal timeout([EnforceRange] unsigned long long milliseconds);
  [NewObject] static AbortSignal _any(sequence<AbortSignal> signals);
  readonly attribute boolean aborted;
  readonly attribute any reason;
  undefined throwIfAborted();

  attribute EventHandler onabort;
};
