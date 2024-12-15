/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

// https://console.spec.whatwg.org/

[ClassString="Console",
 Exposed=*]
namespace console {
  // Logging
  undefined assert(optional boolean condition = false, any... data);
  undefined clear();
  undefined debug(any... messages);
  undefined error(any... messages);
  undefined info(any... messages);
  undefined log(any... messages);
  // undefined table(optional any tabularData, optional sequence<DOMString> properties);
  undefined trace(any... data);
  undefined warn(any... messages);
  // undefined dir(optional any item, optional object? options);
  // undefined dirxml(any... data);

  // Counting
  undefined count(optional DOMString label = "default");
  undefined countReset(optional DOMString label = "default");

  // Grouping
  undefined group(any... data);
  undefined groupCollapsed(any... data);
  undefined groupEnd();

  // Timing
  undefined time(optional DOMString label = "default");
  undefined timeLog(optional DOMString label = "default", any... data);
  undefined timeEnd(optional DOMString label = "default");
};
