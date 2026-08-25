// Copyright (C) 2022 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-temporal.zoneddatetime.prototype.with
description: PreparePartialTemporalFields copies only defined properties of source object
info: |
  4. For each value _property_ of _fieldNames_, do
    a. Let _value_ be ? Get(_fields_, _property_).
    b. If _value_ is not *undefined*, then
        ...
        iii. Perform ! CreateDataPropertyOrThrow(_result_, _property_, _value_).
features: [Temporal]
---*/

const d1 = new Temporal.ZonedDateTime(1_000_000_000_000_000_789n, "UTC");

const d2 = d1.with({ day: 1, hour: 10, year: undefined });

assert.sameValue(d2.year, 2001,
  "only the properties that are present and defined in the plain object are copied (year value)"
);

assert.sameValue(d2.month, 9,
  "only the properties that are present and defined in the plain object are copied (month value)"
);

assert.sameValue(d2.day, 1,
  "only the properties that are present and defined in the plain object are copied (day value)"
);

assert.sameValue(d2.hour, 10,
  "only the properties that are present and defined in the plain object are copied (hour value)"
);
assert.sameValue(d2.minute, 46,
  "only the properties that are present and defined in the plain object are copied (minute value)"
);
assert.sameValue(d2.second, 40,
  "only the properties that are present and defined in the plain object are copied (second value)"
);
assert.sameValue(d2.millisecond, 0,
  "only the properties that are present and defined in the plain object are copied (millisecond value)"
);
assert.sameValue(d2.microsecond, 0,
  "only the properties that are present and defined in the plain object are copied (microsecond value)"
);
assert.sameValue(d2.nanosecond, 789,
  "only the properties that are present and defined in the plain object are copied (nanosecond value)"
);
