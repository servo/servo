// Copyright (C) 2022 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-temporal.plaintime.from
description: Object argument handles leap seconds according to the overflow option.
includes: [temporalHelpers.js]
features: [Temporal]
---*/

for (const options of [undefined, {}, { overflow: "constrain" }]) {
  TemporalHelpers.assertPlainTime(Temporal.PlainTime.from({ hour: 23, minute: 59, second: 60 }, options),
    23, 59, 59, 0, 0, 0);
  TemporalHelpers.assertPlainTime(Temporal.PlainTime.from({ hour: 12, minute: 30, second: 60 }, options),
    12, 30, 59, 0, 0, 0);
  TemporalHelpers.assertPlainTime(Temporal.PlainTime.from({ hour: 23, minute: 59, second: 60, millisecond: 170 }, options),
    23, 59, 59, 170, 0, 0);
}

const options = { overflow: "reject" };
assert.throws(RangeError, () => Temporal.PlainTime.from({ hour: 23, minute: 59, second: 60 }, options));
assert.throws(RangeError, () => Temporal.PlainTime.from({ hour: 12, minute: 30, second: 60 }, options));
assert.throws(RangeError, () => Temporal.PlainTime.from({ hour: 23, minute: 59, second: 60, millisecond: 170 }, options));
