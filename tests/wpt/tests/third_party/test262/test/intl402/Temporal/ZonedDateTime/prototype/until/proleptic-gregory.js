// Copyright (C) 2025 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-temporal.zoneddatetime.prototype.until
description: >
  Check that Gregorian calendar is implemented as proleptic
  (gregory calendar)
features: [Temporal, Intl.Era-monthcode]
includes: [temporalHelpers.js]
---*/

const calendar = "gregory";

const date15821004 = Temporal.ZonedDateTime.from({ year: 1582, monthCode: "M10", day: 4, hour: 12, minute: 34, timeZone: "UTC", calendar });
const date15821007 = Temporal.ZonedDateTime.from({ year: 1582, monthCode: "M10", day: 7, hour: 12, minute: 34, timeZone: "UTC", calendar });
const date15821011 = Temporal.ZonedDateTime.from({ year: 1582, monthCode: "M10", day: 11, hour: 12, minute: 34, timeZone: "UTC", calendar });
const date15821012 = Temporal.ZonedDateTime.from({ year: 1582, monthCode: "M10", day: 12, hour: 12, minute: 34, timeZone: "UTC", calendar });
const date15821015 = Temporal.ZonedDateTime.from({ year: 1582, monthCode: "M10", day: 15, hour: 12, minute: 34, timeZone: "UTC", calendar });
TemporalHelpers.assertDuration(
  date15821004.until(date15821007, { largestUnit: "days" }),
  0, 0, 0, 3, 0, 0, 0, 0, 0, 0,
  "1582-10-04 and 1582-10-07");
TemporalHelpers.assertDuration(
  date15821015.until(date15821012, { largestUnit: "days" }),
  0, 0, 0, -3, 0, 0, 0, 0, 0, 0,
  "1582-10-15 and 1582-10-12");
TemporalHelpers.assertDuration(
  date15821004.until(date15821011, { largestUnit: "weeks" }),
  0, 0, 1, 0, 0, 0, 0, 0, 0, 0,
  "1582-10-04 and 1582-10-11")
TemporalHelpers.assertDuration(
  date15821011.until(date15821004, { largestUnit: "weeks" }),
  0, 0, -1, 0, 0, 0, 0, 0, 0, 0,
  "1582-10-11 and 1582-10-04")
