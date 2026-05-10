// Copyright (C) 2021 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-temporal.duration.prototype.tostring
description: Plural units are accepted as well for the smallestUnit option
includes: [temporalHelpers.js]
features: [Temporal]
---*/

const duration = new Temporal.Duration(1, 2, 3, 4, 5, 6, 7, 987, 654, 321);
const validUnits = [
  "second",
  "millisecond",
  "microsecond",
  "nanosecond",
];
TemporalHelpers.checkPluralUnitsAccepted((smallestUnit) => duration.toString({ smallestUnit }), validUnits);
