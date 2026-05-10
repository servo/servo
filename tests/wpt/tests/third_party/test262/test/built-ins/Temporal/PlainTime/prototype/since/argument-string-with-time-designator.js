// Copyright (C) 2022 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-temporal.plaintime.prototype.since
description: ISO 8601 time designator "T" allowed at the start of PlainTime strings
includes: [temporalHelpers.js]
features: [Temporal, arrow-function]
---*/

const instance = new Temporal.PlainTime(1, 0, 0, 0, 0, 1);
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
  const result = instance.since(arg);
  TemporalHelpers.assertDuration(result, 0, 0, 0, 0, 0, 30, 0, 0, 0, 1, `T prefix is accepted: ${arg}`);
});
