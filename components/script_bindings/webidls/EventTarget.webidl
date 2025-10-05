/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */
/*
 * https://dom.spec.whatwg.org/#interface-eventtarget
 */

partial interface EventTarget {
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

// https://dom.spec.whatwg.org/#dictdef-eventlisteneroptions
dictionary EventListenerOptions {
  boolean capture = false;
};

// https://dom.spec.whatwg.org/#dictdef-addeventlisteneroptions
dictionary AddEventListenerOptions : EventListenerOptions {
  boolean passive;
  boolean once = false;
  AbortSignal signal;
};
