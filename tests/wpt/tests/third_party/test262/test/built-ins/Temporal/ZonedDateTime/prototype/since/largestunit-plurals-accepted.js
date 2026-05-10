// Copyright (C) 2021 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-temporal.zoneddatetime.prototype.since
description: Plural units are accepted as well for the largestUnit option
includes: [temporalHelpers.js]
features: [Temporal]
---*/

const earlier = new Temporal.ZonedDateTime(1_000_000_000_987_654_321n, "UTC");
const later = new Temporal.ZonedDateTime(1_086_403_661_988_655_322n, "UTC");
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
TemporalHelpers.checkPluralUnitsAccepted((largestUnit) => later.since(earlier, { largestUnit }), validUnits);
