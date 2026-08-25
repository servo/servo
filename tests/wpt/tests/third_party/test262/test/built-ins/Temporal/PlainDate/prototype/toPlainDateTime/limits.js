// Copyright (C) 2022 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-temporal.plaindate.prototype.toplaindatetime
description: Checking limits of representable PlainDateTime
features: [Temporal]
includes: [temporalHelpers.js]
---*/

const midnight = new Temporal.PlainTime(0, 0);
const firstNs = new Temporal.PlainTime(0, 0, 0, 0, 0, 1);
const lastNs = new Temporal.PlainTime(23, 59, 59, 999, 999, 999);
const min = new Temporal.PlainDate(-271821, 4, 19);
const max = new Temporal.PlainDate(275760, 9, 13);

assert.throws(
  RangeError,
  () => min.toPlainDateTime(midnight),
  "Cannot go below representable limit for PlainDateTime"
);

TemporalHelpers.assertPlainDateTime(
  max.toPlainDateTime(midnight),
  275760, 9, "M09", 13, 0, 0, 0, 0, 0, 0,
  "Midnight on maximal representable PlainDate"
);

TemporalHelpers.assertPlainDateTime(
  min.toPlainDateTime(firstNs),
  -271821, 4, "M04", 19, 0, 0, 0, 0, 0, 1,
  "Computing the minimum (earliest) representable PlainDateTime"
);

TemporalHelpers.assertPlainDateTime(
  max.toPlainDateTime(lastNs),
  275760, 9, "M09", 13, 23, 59, 59, 999, 999, 999,
  "Computing the maximum (latest) representable PlainDateTime"
);
