// Copyright 2018 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-Intl.ListFormat
description: Checks handling of invalid value for the localeMatcher option to the ListFormat constructor.
info: |
    Intl.ListFormat ( [ locales [ , options ] ] )
    12. Let matcher be ? GetOption(options, "localeMatcher", "string", « "lookup", "best fit" », "best fit").
features: [Intl.ListFormat]
---*/

const invalidOptions = [
  null,
  1,
  "",
  "Lookup",
  "LOOKUP",
  "lookup\0",
  "Best fit",
  "BEST FIT",
  "best\u00a0fit",
];

for (const localeMatcher of invalidOptions) {
  assert.throws(RangeError, function() {
    new Intl.ListFormat([], { localeMatcher });
  }, `${localeMatcher} is an invalid localeMatcher option value`);
}
