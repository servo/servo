/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */
/*
 * For more information on this interface please see
 * https://dom.spec.whatwg.org/#event
 */

[Constructor(DOMString type, optional EventInit eventInitDict), Exposed=(Window,Worker)]
interface Event {
  [Pure]
  readonly attribute DOMString type;
  readonly attribute EventTarget? target;
  readonly attribute EventTarget? currentTarget;

  const unsigned short NONE = 0;
  const unsigned short CAPTURING_PHASE = 1;
  const unsigned short AT_TARGET = 2;
  const unsigned short BUBBLING_PHASE = 3;
  readonly attribute unsigned short eventPhase;

  void stopPropagation();
  void stopImmediatePropagation();

  [Pure]
  readonly attribute boolean bubbles;
  [Pure]
  readonly attribute boolean cancelable;
  void preventDefault();
  [Pure]
  readonly attribute boolean defaultPrevented;

  [Unforgeable]
  readonly attribute boolean isTrusted;
  [Constant]
  readonly attribute DOMTimeStamp timeStamp;

  void initEvent(DOMString type, boolean bubbles, boolean cancelable);
};

dictionary EventInit {
  boolean bubbles = false;
  boolean cancelable = false;
};
