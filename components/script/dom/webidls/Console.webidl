/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

// https://console.spec.whatwg.org/

[ClassString="Console",
 Exposed=(Window,Worker,Worklet),
 ProtoObjectHack]
namespace console {
  // Logging
  void log(DOMString... messages);
  void debug(DOMString... messages);
  void info(DOMString... messages);
  void warn(DOMString... messages);
  void error(DOMString... messages);
  void assert(boolean condition, optional DOMString message);
  void clear();

  // Grouping
  void group(DOMString... data);
  void groupCollapsed(DOMString... data);
  void groupEnd();

  // Timing
  void time(DOMString message);
  void timeEnd(DOMString message);
};
