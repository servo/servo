// Copyright (C) 2022 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-temporal.plaintime.from
description: Plain object argument is supported and ignores plural properties
includes: [temporalHelpers.js]
features: [Temporal]
---*/

TemporalHelpers.assertPlainTime(Temporal.PlainTime.from({ hour: 15, minute: 23 }),
  15, 23, 0, 0, 0, 0);
TemporalHelpers.assertPlainTime(Temporal.PlainTime.from({ minute: 30, microsecond: 555 }),
  0, 30, 0, 0, 555, 0);
TemporalHelpers.assertPlainTime(Temporal.PlainTime.from({ year: 2019, month: 10, day: 1, hour: 14, minute: 20, second: 36 }),
  14, 20, 36, 0, 0, 0);
TemporalHelpers.assertPlainTime(Temporal.PlainTime.from({ hours: 2, minute: 30, microsecond: 555 }),
  0, 30, 0, 0, 555, 0);

assert.throws(TypeError, () => Temporal.PlainTime.from({}));
assert.throws(TypeError, () => Temporal.PlainTime.from({ minutes: 12 }));
