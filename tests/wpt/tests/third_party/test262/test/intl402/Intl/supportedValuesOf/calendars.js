// Copyright (C) 2021 André Bargull. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-intl.supportedvaluesof
description: >
  The returned "calendar" values are sorted, unique, and match the type production.
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
includes: [compareArray.js]
features: [Intl-enumeration, Intl.Locale, Array.prototype.includes]
---*/

const calendars = Intl.supportedValuesOf("calendar");

assert(Array.isArray(calendars), "Returns an Array object.");
assert.sameValue(Object.getPrototypeOf(calendars), Array.prototype,
                 "The array prototype is Array.prototype");

const otherCalendars = Intl.supportedValuesOf("calendar");
assert.notSameValue(otherCalendars, calendars,
                    "Returns a new array object for each call.");

assert.compareArray(calendars, otherCalendars.sort(),
                    "The array is sorted.");

assert.sameValue(new Set(calendars).size, calendars.length,
                 "The array doesn't contain duplicates.");

// https://unicode.org/reports/tr35/tr35.html#Unicode_locale_identifier
const typeRE = /^[a-z0-9]{3,8}(-[a-z0-9]{3,8})*$/;
for (let calendar of calendars) {
  assert(typeRE.test(calendar), `${calendar} matches the 'type' production`);
}

for (let calendar of calendars) {
  assert.sameValue(new Intl.Locale("und", {calendar}).calendar, calendar,
                   `${calendar} is canonicalised`);
}

assert(calendars.includes("gregory"), "Includes the Gregorian calendar.");
