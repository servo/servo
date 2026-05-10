// Copyright (C) 2021 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-temporal.plaintime.prototype.round
description: Plural units are accepted as well for the smallestUnit option
includes: [temporalHelpers.js]
features: [Temporal, arrow-function]
---*/

const time = new Temporal.PlainTime(12, 34, 56, 789, 999, 999);
const validUnits = [
  "hour",
  "minute",
  "second",
  "millisecond",
  "microsecond",
  "nanosecond",
];
TemporalHelpers.checkPluralUnitsAccepted((smallestUnit) => time.round({ smallestUnit }), validUnits);
TemporalHelpers.checkPluralUnitsAccepted((smallestUnit) => time.round(smallestUnit), validUnits);
