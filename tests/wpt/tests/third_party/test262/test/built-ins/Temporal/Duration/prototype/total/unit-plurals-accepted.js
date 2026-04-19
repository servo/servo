// Copyright (C) 2021 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-temporal.duration.prototype.total
description: Plural units are accepted as well for the unit option
includes: [temporalHelpers.js]
features: [Temporal]
---*/

const duration = new Temporal.Duration(1, 2, 3, 4, 5, 6, 7, 987, 654, 321);
const relativeTo = new Temporal.PlainDate(2000, 1, 1);
const validUnits = [
  "year",
  "month",
  "week",
  "day",
  "hour",
  "minute",
  "second",
  "millisecond",
  "microsecond",
  "nanosecond",
];
TemporalHelpers.checkPluralUnitsAccepted((unit) => duration.total({ unit, relativeTo }), validUnits);
