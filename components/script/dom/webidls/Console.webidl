/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

// https://console.spec.whatwg.org/

[ClassString="Console",
 Exposed=(Window,Worker,Worklet),
 ProtoObjectHack]
namespace console {
  // Logging
  undefined log(any... messages);
  undefined debug(any... messages);
  undefined info(any... messages);
  undefined warn(any... messages);
  undefined error(any... messages);
  undefined assert(boolean condition, optional any message);
  undefined clear();

  // Counting
  undefined count(optional DOMString label = "default");
  undefined countReset(optional DOMString label = "default");

  // Grouping
  undefined group(any... data);
  undefined groupCollapsed(any... data);
  undefined groupEnd();

  // Timing
  undefined time(DOMString message);
  undefined timeEnd(DOMString message);
};
