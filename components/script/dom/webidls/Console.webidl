/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

// https://console.spec.whatwg.org/

[ClassString="Console",
 Exposed=(Window,Worker,Worklet),
 ProtoObjectHack]
namespace console {
  // Logging
  undefined log(DOMString... messages);
  undefined debug(DOMString... messages);
  undefined info(DOMString... messages);
  undefined warn(DOMString... messages);
  undefined error(DOMString... messages);
  undefined assert(boolean condition, optional DOMString message);
  undefined clear();

  // Grouping
  undefined group(DOMString... data);
  undefined groupCollapsed(DOMString... data);
  undefined groupEnd();

  // Timing
  undefined time(DOMString message);
  undefined timeEnd(DOMString message);
};
