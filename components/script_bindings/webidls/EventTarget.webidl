/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */
/*
 * https://dom.spec.whatwg.org/#interface-eventtarget
 */

[Exposed=(Window,Worker,Worklet,DissimilarOriginWindow)]
interface EventTarget {
  [Throws] constructor();
  undefined addEventListener(
    DOMString type,
    EventListener? callback,
    optional (AddEventListenerOptions or boolean) options = {}
  );

  undefined removeEventListener(
    DOMString type,
    EventListener? callback,
    optional (EventListenerOptions or boolean) options = {}
  );

  [Throws]
  boolean dispatchEvent(Event event);
};

dictionary EventListenerOptions {
  boolean capture = false;
};

dictionary AddEventListenerOptions : EventListenerOptions {
  // boolean passive = false;
  boolean once = false;
};
