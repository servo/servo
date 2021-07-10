/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */
/*
 * For more information on this interface please see
 * https://dom.spec.whatwg.org/#event
 */

[Exposed=(Window,Worker)]
interface Event {
  [Throws] constructor(DOMString type, optional EventInit eventInitDict = {});
  [Pure]
  readonly attribute DOMString type;
  readonly attribute EventTarget? target;
  readonly attribute EventTarget? srcElement;
  readonly attribute EventTarget? currentTarget;

  const unsigned short NONE = 0;
  const unsigned short CAPTURING_PHASE = 1;
  const unsigned short AT_TARGET = 2;
  const unsigned short BUBBLING_PHASE = 3;
  readonly attribute unsigned short eventPhase;

  void stopPropagation();
  attribute boolean cancelBubble;
  void stopImmediatePropagation();

  [Pure]
  readonly attribute boolean bubbles;
  [Pure]
  readonly attribute boolean cancelable;
  attribute boolean returnValue;  // historical
  void preventDefault();
  [Pure]
  readonly attribute boolean defaultPrevented;

  [Unforgeable]
  readonly attribute boolean isTrusted;
  [Constant]
  readonly attribute DOMHighResTimeStamp timeStamp;

  void initEvent(DOMString type, optional boolean bubbles = false, optional boolean cancelable = false);
};

dictionary EventInit {
  boolean bubbles = false;
  boolean cancelable = false;
};
