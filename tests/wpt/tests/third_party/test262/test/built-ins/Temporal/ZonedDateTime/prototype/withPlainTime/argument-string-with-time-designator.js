// Copyright (C) 2022 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-temporal.zoneddatetime.prototype.withplaintime
description: ISO 8601 time designator "T" allowed at the start of PlainTime strings
features: [Temporal, arrow-function]
---*/

const instance = new Temporal.ZonedDateTime(1_000_000_000_000_000_000n, "UTC");
const validStrings = [
  "T00:30",
  "t00:30",
  "T0030",
  "t0030",
  "T00:30:00",
  "t00:30:00",
  "T003000",
  "t003000",
  "T00:30:00.000000000",
  "t00:30:00.000000000",
  "T003000.000000000",
  "t003000.000000000",
];
validStrings.forEach((arg) => {
  const result = instance.withPlainTime(arg);
  assert.sameValue(result.epochNanoseconds, 999995400_000_000_000n, `T prefix is accepted: ${arg}`);
});
