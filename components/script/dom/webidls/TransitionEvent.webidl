/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */
/*
 * For more information on this interface please see
 * https://dom.spec.whatwg.org/#event
 */

[Constructor(DOMString type, optional TransitionEventInit transitionEventInitDict),
 Exposed=Window]
interface TransitionEvent : Event {
  readonly attribute DOMString          propertyName;
  readonly attribute float              elapsedTime;
  readonly attribute DOMString          pseudoElement;
};

dictionary TransitionEventInit : EventInit {
  DOMString propertyName = "";
  float elapsedTime = 0.0;
  DOMString pseudoElement = "";
};
