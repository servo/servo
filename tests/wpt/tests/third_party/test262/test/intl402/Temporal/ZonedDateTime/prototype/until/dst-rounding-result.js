// Copyright (C) 2024 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: sec-temporal.zoneddatetime.prototype.until
description: >
    Rounding the resulting duration takes the time zone's UTC offset shifts
    into account
includes: [temporalHelpers.js]
features: [Temporal]
---*/

// Difference rounding (nearest day) is DST-aware
let start = Temporal.PlainDateTime.from("2000-04-04T02:30").toZonedDateTime("America/Vancouver");
let end = Temporal.PlainDateTime.from("2000-04-01T14:15").toZonedDateTime("America/Vancouver");
let diff = start.until(end, {
  smallestUnit: "days",
  roundingMode: "halfExpand"
});
TemporalHelpers.assertDuration(
  diff,
  0, 0, 0, -3, 0, 0, 0, 0, 0, 0,
  "Difference rounding (nearest day) is DST-aware");

// Difference rounding (ceil day) is DST-aware
diff = start.until(end, {
  smallestUnit: "days",
  roundingMode: "ceil"
});
TemporalHelpers.assertDuration(
  diff,
  0, 0, 0, -2, 0, 0, 0, 0, 0, 0,
  "Difference rounding (ceil day) is DST-aware");

// Difference rounding (trunc day) is DST-aware
diff = start.until(end, {
  smallestUnit: "days",
  roundingMode: "trunc"
});
TemporalHelpers.assertDuration(
  diff,
  0, 0, 0, -2, 0, 0, 0, 0, 0, 0,
  "Difference rounding (trunc day) is DST-aware");

// Difference rounding (floor day) is DST-aware
diff = start.until(end, {
  smallestUnit: "days",
  roundingMode: "floor"
});
TemporalHelpers.assertDuration(
  diff,
  0, 0, 0, -3, 0, 0, 0, 0, 0, 0,
  "Difference rounding (floor day) is DST-aware");

// Difference rounding (nearest hour) is DST-aware
diff = start.until(end, {
  largestUnit: "days",
  smallestUnit: "hours",
  roundingMode: "halfExpand"
});
TemporalHelpers.assertDuration(
  diff,
  0, 0, 0, -2, -12, 0, 0, 0, 0, 0,
  "Difference rounding (nearest hour) is DST-aware");

// Difference rounding (ceil hour) is DST-aware
diff = start.until(end, {
  largestUnit: "days",
  smallestUnit: "hours",
  roundingMode: "ceil"
});
TemporalHelpers.assertDuration(
  diff,
  0, 0, 0, -2, -12, 0, 0, 0, 0, 0,
  "Difference rounding (ceil hour) is DST-aware");

// Difference rounding (trunc hour) is DST-aware
diff = start.until(end, {
  largestUnit: "days",
  smallestUnit: "hours",
  roundingMode: "trunc"
});
TemporalHelpers.assertDuration(
  diff,
  0, 0, 0, -2, -12, 0, 0, 0, 0, 0,
  "Difference rounding (trunc hour) is DST-aware");

// Difference rounding (floor hour) is DST-aware
diff = start.until(end, {
  largestUnit: "days",
  smallestUnit: "hours",
  roundingMode: "floor"
});
TemporalHelpers.assertDuration(
  diff,
  0, 0, 0, -2, -13, 0, 0, 0, 0, 0,
  "Difference rounding (floor hour) is DST-aware");

// Difference when date portion ends inside a DST-skipped period
start = Temporal.PlainDateTime.from("2000-04-01T02:30").toZonedDateTime("America/Vancouver");
end = Temporal.PlainDateTime.from("2000-04-02T03:15").toZonedDateTime("America/Vancouver");
diff = start.until(end, { largestUnit: "days" });
TemporalHelpers.assertDuration(
  diff,
  0, 0, 0, 0, 23, 45, 0, 0, 0, 0,
  "Difference when date portion ends inside a DST-skipped period");

// Difference when date portion ends inside day skipped by Samoa's 24hr 2011 transition
end = Temporal.PlainDateTime.from("2011-12-31T05:00").toZonedDateTime("Pacific/Apia");
start = Temporal.PlainDateTime.from("2011-12-28T10:00").toZonedDateTime("Pacific/Apia");
diff = start.until(end, { largestUnit: "days" });
TemporalHelpers.assertDuration(
  diff,
  0, 0, 0, 1, 19, 0, 0, 0, 0, 0,
  "Difference when date portion ends inside day skipped by Samoa's 24hr 2011 transition");

// Rounding up to hours causes one more day of overflow (positive)
start = Temporal.ZonedDateTime.from("2020-01-01T00:00-08:00[-08:00]");
end = Temporal.ZonedDateTime.from("2020-01-03T23:59-08:00[-08:00]");
diff = start.until(end, {
  largestUnit: "days",
  smallestUnit: "hours",
  roundingMode: "halfExpand"
});
TemporalHelpers.assertDuration(
  diff,
  0, 0, 0, 3, 0, 0, 0, 0, 0, 0,
  "Rounding up to hours causes one more day of overflow (positive)");

// Rounding up to hours causes one more day of overflow (negative)
start = Temporal.ZonedDateTime.from("2020-01-01T00:00-08:00[-08:00]");
end = Temporal.ZonedDateTime.from("2020-01-03T23:59-08:00[-08:00]");
diff = end.until(start, {
  largestUnit: "days",
  smallestUnit: "hours",
  roundingMode: "halfExpand"
});
TemporalHelpers.assertDuration(
  diff,
  0, 0, 0, -3, 0, 0, 0, 0, 0, 0,
  "Rounding up to hours causes one more day of overflow (negative)");

// Based on a test case by Adam Shaw

// Month-only part of duration lands on skipped DST hour, should not cause
// disambiguation
start = new Temporal.ZonedDateTime(
  950868000_000_000_000n /* = 2000-02-18T10Z */,
  "America/Vancouver"); /* = 2000-02-18T02-08 in local time */
end = new Temporal.ZonedDateTime(
  954709200_000_000_000n /* = 2000-04-02T21Z */,
  "America/Vancouver"); /* = 2000-04-02T14-07 in local time */

let duration = start.until(end, { largestUnit: "months" });
TemporalHelpers.assertDuration(duration, 0, 1, 0, 15, 11, 0, 0, 0, 0, 0,
  "1-month rounding window is shortened by DST");




// Month-only part of duration lands on skipped DST hour, should not cause
// disambiguation
start = new Temporal.ZonedDateTime(
  951991200_000_000_000n /* = 2000-03-02T10Z */,
  "America/Vancouver"); /* = 2000-03-02T02-08 in local time */
end = new Temporal.ZonedDateTime(
  956005200_000_000_000n /* = 2000-04-17T21Z */,
  "America/Vancouver"); /* = 2000-04-17T14-07 in local time */

duration = start.until(end, { largestUnit: "months" });
TemporalHelpers.assertDuration(duration, 0, 1, 0, 15, 12, 0, 0, 0, 0, 0,
  "1-month rounding window is not shortened by DST");

