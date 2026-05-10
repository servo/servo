// Copyright (C) 2022 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-temporal.duration.prototype.with
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

const d = new Temporal.Duration(9, 8, 7, 6, 5, 4, 3, 2, 1, 0);

TemporalHelpers.assertDuration(
  d.with({ minutes: 11, hours: 6, months: undefined }),
  9, 8, 7, 6, 6, 11, 3, 2, 1, 0,
  "only the properties that are present and defined in the plain object are copied"
);
