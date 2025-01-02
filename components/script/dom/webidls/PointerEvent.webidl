/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

// https://w3c.github.io/pointerevents/#pointerevent-interface
[Exposed=Window]
interface PointerEvent : MouseEvent
{
  constructor(DOMString type, optional PointerEventInit eventInitDict = {});

  readonly attribute long pointerId;

  readonly attribute long width;
  readonly attribute long height;
  readonly attribute float pressure;
  readonly attribute float tangentialPressure;
  readonly attribute long tiltX;
  readonly attribute long tiltY;
  readonly attribute long twist;
  readonly attribute double altitudeAngle;
  readonly attribute double azimuthAngle;

  readonly attribute DOMString pointerType;
  readonly attribute boolean isPrimary;

  sequence<PointerEvent> getCoalescedEvents();
  sequence<PointerEvent> getPredictedEvents();
};

dictionary PointerEventInit : MouseEventInit
{
  long pointerId = 0;
  long width = 1;
  long height = 1;
  float pressure = 0;
  float tangentialPressure = 0;
  long tiltX;
  long tiltY;
  long twist = 0;
  double altitudeAngle;
  double azimuthAngle;
  DOMString pointerType = "";
  boolean isPrimary = false;
  sequence<PointerEvent> coalescedEvents = [];
  sequence<PointerEvent> predictedEvents = [];
};
