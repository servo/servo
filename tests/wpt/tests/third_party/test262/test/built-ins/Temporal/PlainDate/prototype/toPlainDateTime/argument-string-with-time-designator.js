// Copyright (C) 2022 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-temporal.plaindate.prototype.toplaindatetime
description: ISO 8601 time designator "T" allowed at the start of PlainTime strings
includes: [temporalHelpers.js]
features: [Temporal, arrow-function]
---*/

const instance = new Temporal.PlainDate(2000, 1, 1);
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
  const result = instance.toPlainDateTime(arg);
  TemporalHelpers.assertPlainDateTime(result, 2000, 1, "M01", 1, 0, 30, 0, 0, 0, 0, `T prefix is accepted: ${arg}`);
});
