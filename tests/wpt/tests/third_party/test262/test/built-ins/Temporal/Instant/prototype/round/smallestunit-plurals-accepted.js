// Copyright (C) 2021 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-temporal.instant.prototype.round
description: Plural units are accepted as well for the smallestUnit option
includes: [temporalHelpers.js]
features: [Temporal, arrow-function]
---*/

const instant = new Temporal.Instant(1_000_000_000_987_654_321n);
const validUnits = [
  "hour",
  "minute",
  "second",
  "millisecond",
  "microsecond",
  "nanosecond",
];
TemporalHelpers.checkPluralUnitsAccepted((smallestUnit) => instant.round({ smallestUnit }), validUnits);
TemporalHelpers.checkPluralUnitsAccepted((smallestUnit) => instant.round(smallestUnit), validUnits);
