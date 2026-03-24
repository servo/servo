/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

// https://html.spec.whatwg.org/multipage/#the-closewatcher-interface
[Exposed=Window]
interface CloseWatcher : EventTarget {
  [Throws] constructor(optional CloseWatcherOptions options = {});

  undefined requestClose();
  undefined close();
  undefined destroy();

  attribute EventHandler oncancel;
  attribute EventHandler onclose;
};

dictionary CloseWatcherOptions {
  AbortSignal signal;
};
