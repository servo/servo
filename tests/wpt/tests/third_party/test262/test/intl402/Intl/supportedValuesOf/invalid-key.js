// Copyright (C) 2021 André Bargull. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-intl.supportedvaluesof
description: >
  Intl.supportedValuesOf throws a RangeError if the key is invalid.
info: |
  Intl.supportedValuesOf ( key )

  1. Let key be ? ToString(key).
  ...
  8. Else,
    a. Throw a RangeError exception.
  ...
features: [Intl-enumeration]
---*/

const invalidKeys = [
  // Empty string is invalid.
  "",

  // Various unsupported keys.
  "hourCycle", "locale", "language", "script", "region",

  // Plural form of supported keys not valid.
  "calendars", "collations", "currencies", "numberingSystems", "timeZones", "units",

  // Wrong case for supported keys.
  "CALENDAR", "Collation", "Currency", "numberingsystem", "timezone", "UNIT",

  // NUL character must be handled correctly.
  "calendar\0",

  // Non-string cases.
  undefined, null, false, true, NaN, 0, Math.PI, 123n, {}, [],
];

for (let key of invalidKeys) {
  assert.throws(RangeError, function() {
    Intl.supportedValuesOf(key);
  }, "key: " + key);
}
