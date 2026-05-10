// Copyright (C) 2021 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-temporal.plainyearmonth.prototype.until
description: Plural units are accepted as well for the largestUnit option
includes: [temporalHelpers.js]
features: [Temporal]
---*/

const earlier = new Temporal.PlainYearMonth(2000, 5);
const later = new Temporal.PlainYearMonth(2001, 6);
const validUnits = [
  "year",
  "month",
];
TemporalHelpers.checkPluralUnitsAccepted((largestUnit) => earlier.until(later, { largestUnit }), validUnits);
