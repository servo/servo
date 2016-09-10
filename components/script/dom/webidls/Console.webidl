/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */
/*
 * References:
 *   MDN Docs - https://developer.mozilla.org/en-US/docs/Web/API/console
 *   Draft Spec - https://sideshowbarker.github.io/console-spec/
 *
 * © Copyright 2014 Mozilla Foundation.
 */

[ClassString="Console",
 Exposed=(Window,Worker),
 ProtoObjectHack]
namespace console {
  // These should be DOMString message, DOMString message2, ...
  void log(DOMString... messages);
  void debug(DOMString... messages);
  void info(DOMString... messages);
  void warn(DOMString... messages);
  void error(DOMString... messages);
  void assert(boolean condition, optional DOMString message);
  void time(DOMString message);
  void timeEnd(DOMString message);
};
