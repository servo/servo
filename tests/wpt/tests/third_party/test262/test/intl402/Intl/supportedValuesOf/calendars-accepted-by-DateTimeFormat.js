// Copyright (C) 2021 André Bargull. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-intl.supportedvaluesof
description: >
  The returned "calendar" values can be used with DateTimeFormat.
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
features: [Intl-enumeration, Array.prototype.includes]
---*/

const calendars = Intl.supportedValuesOf("calendar");

for (let calendar of calendars) {
  let obj = new Intl.DateTimeFormat("en", {calendar});
  assert.sameValue(obj.resolvedOptions().calendar, calendar,
                   `${calendar} is supported by DateTimeFormat`);
}

for (let calendar of allCalendars()) {
  let obj = new Intl.DateTimeFormat("en", {calendar});
  if (obj.resolvedOptions().calendar === calendar) {
    assert(calendars.includes(calendar),
           `${calendar} supported but not returned by supportedValuesOf`);
  } else {
    assert(!calendars.includes(calendar),
           `${calendar} not supported but returned by supportedValuesOf`);
  }
}
