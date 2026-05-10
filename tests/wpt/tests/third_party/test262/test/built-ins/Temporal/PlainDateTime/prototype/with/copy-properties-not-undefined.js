// Copyright (C) 2022 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-temporal.plaindatetime.prototype.with
description: PreparePartialTemporalFields copies only defined properties of source object
info: |
  4. For each value _property_ of _fieldNames_, do
    a. Let _value_ be ? Get(_fields_, _property_).
    b. If _value_ is not *undefined*, then
        ...
        iii. Perform ! CreateDataPropertyOrThrow(_result_, _property_, _value_).
includes: [temporalHelpers.js]
features: [Temporal]
---*/

const plainDateTime = new Temporal.PlainDateTime(2006, 1, 24, 11, 42, 58);

TemporalHelpers.assertPlainDateTime(plainDateTime.with({ day: 8, hour: 10, year: undefined }),
  2006, 1, "M01", 8, 10, 42, 58, 0, 0, 0,
  "only the properties that are present and defined in the plain object are copied"
);
// See options-empty.js for {} and () => {}
