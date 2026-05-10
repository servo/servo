// Copyright (C) 2021 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-temporal.duration.prototype.total
description: Plural units are accepted as well for the shorthand for the unit option
includes: [temporalHelpers.js]
features: [Temporal, arrow-function]
---*/

const duration = new Temporal.Duration(0, 0, 0, 4, 5, 6, 7, 987, 654, 321);
const validUnits = [
  "day",
  "hour",
  "minute",
  "second",
  "millisecond",
  "microsecond",
  "nanosecond",
];
TemporalHelpers.checkPluralUnitsAccepted((unit) => duration.total(unit), validUnits);
