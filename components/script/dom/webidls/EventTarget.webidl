/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */
/*
 * https://dom.spec.whatwg.org/#interface-eventtarget
 */

[Abstract, Exposed=(Window,Worker,Worklet)]
interface EventTarget {
  void addEventListener(
    DOMString type,
    EventListener? callback,
    optional (AddEventListenerOptions or boolean) options
  );

  void removeEventListener(
    DOMString type,
    EventListener? callback,
    optional (EventListenerOptions or boolean) options
  );

  [Throws]
  boolean dispatchEvent(Event event);
};

dictionary EventListenerOptions {
  boolean capture = false;
};

dictionary AddEventListenerOptions : EventListenerOptions {
  // boolean passive = false;
  // boolean once = false;
};
