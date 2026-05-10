// Copyright (C) 2022 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-temporal.plaindatetime.from
description: Checking limits of representable PlainDateTime
features: [Temporal]
includes: [temporalHelpers.js]
---*/

["reject", "constrain"].forEach((overflow) => {
  assert.throws(
    RangeError,
    () => Temporal.PlainDateTime.from({ year: -271821, month: 4, day: 19 }, { overflow }),
    `negative out of bounds (plain object, overflow = ${overflow})`
  );
  assert.throws(
    RangeError,
    () => Temporal.PlainDateTime.from({ year: 275760, month: 9, day: 14 }, { overflow }),
    `positive out of bounds (plain object, overflow = ${overflow})`
  );
});

TemporalHelpers.assertPlainDateTime(
  Temporal.PlainDateTime.from({ year: -271821, month: 4, day: 19, nanosecond: 1 }),
  -271821, 4, "M04", 19, 0, 0, 0, 0, 0, 1,
  "construct from property bag (negative boundary)"
);

TemporalHelpers.assertPlainDateTime(
  Temporal.PlainDateTime.from(
    {
      year: 275760,
      month: 9,
      day: 13,
      hour: 23,
      minute: 59,
      second: 59,
      millisecond: 999,
      microsecond: 999,
      nanosecond: 999
    }
  ),
  275760, 9, "M09", 13, 23, 59, 59, 999, 999, 999,
  "construct from property bag (positive boundary)"
);

assert.throws(
  RangeError,
  () => Temporal.PlainDateTime.from("-271821-04-19T00:00"),
  "out-of-bounds ISO string (negative case)"
);

assert.throws(
  RangeError,
  () => Temporal.PlainDateTime.from("+275760-09-14T00:00"),
  "out-of-bounds ISO string (positive case)"
);

TemporalHelpers.assertPlainDateTime(
  Temporal.PlainDateTime.from("-271821-04-19T00:00:00.000000001"),
  -271821, 4, "M04", 19, 0, 0, 0, 0, 0, 1,
  "boundary ISO string (negative case)"
);

TemporalHelpers.assertPlainDateTime(
  Temporal.PlainDateTime.from("+275760-09-13T23:59:59.999999999"),
  275760, 9, "M09", 13, 23, 59, 59, 999, 999, 999,
  "boundary ISO string (positive case)"
);
