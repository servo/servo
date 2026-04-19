// Copyright (C) 2021 André Bargull. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-intl.supportedvaluesof
description: >
  Input key is coerced with ToString.
info: |
  Intl.supportedValuesOf ( key )

  1. Let key be ? ToString(key).
  2. If key is "calendar", then
    a. Let list be ! AvailableCalendars( ).
  ...
  9. Return ! CreateArrayFromList( list ).
includes: [compareArray.js]
features: [Intl-enumeration]
---*/

const calendars = Intl.supportedValuesOf("calendar");

// ToString on a String object.
assert.compareArray(Intl.supportedValuesOf(new String("calendar")), calendars);

// ToString on a plain object.
let obj = {
  toString() {
    return "calendar";
  }
};
assert.compareArray(Intl.supportedValuesOf(obj), calendars);

// ToString() of a symbol throws a TypeError.
assert.throws(TypeError, function() {
  Intl.supportedValuesOf(Symbol());
});
