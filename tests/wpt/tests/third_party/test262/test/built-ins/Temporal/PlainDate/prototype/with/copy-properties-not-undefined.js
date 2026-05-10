// Copyright (C) 2022 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-temporal.plaindate.prototype.with
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

const plainDate = new Temporal.PlainDate(2006, 1, 24);

TemporalHelpers.assertPlainDate(plainDate.with({ day: 1, year: undefined }),
  2006, 1, "M01", 1,
  "only the properties that are present and defined in the plain object are copied"
);
