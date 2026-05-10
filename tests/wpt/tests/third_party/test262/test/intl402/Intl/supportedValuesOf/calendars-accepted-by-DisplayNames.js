// Copyright (C) 2021 André Bargull. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-intl.supportedvaluesof
description: >
  The returned "calendar" values can be used with DisplayNames.
info: |
  Intl.supportedValuesOf ( key )

  1. Let key be ? ToString(key).
  2. If key is "calendar", then
    a. Let list be ! AvailableCalendars( ).
  ...
  9. Return ! CreateArrayFromList( list ).

  AvailableCalendars ( )
    The AvailableCalendars abstract operation returns a List, ordered as if an
    Array of the same values had been sorted using %Array.prototype.sort% using
    undefined as comparefn, that contains unique calendar types identifying the
    calendars for which the implementation provides the functionality of
    Intl.DateTimeFormat objects. The list must include "gregory".
includes: [testIntl.js]
locale: [en]
features: [Intl-enumeration, Intl.DisplayNames-v2, Array.prototype.includes]
---*/

const calendars = Intl.supportedValuesOf("calendar");

const obj = new Intl.DisplayNames("en", {type: "calendar", fallback: "none"});

for (let calendar of calendars) {
  assert.sameValue(typeof obj.of(calendar), "string",
                   `${calendar} is supported by DisplayNames`);
}

for (let calendar of allCalendars()) {
  if (typeof obj.of(calendar) !== "string") {
    assert(!calendars.includes(calendar),
           `${calendar} not supported but returned by supportedValuesOf`);
  }
}
