// Copyright (C) 2025 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-temporal.plaindatetime.prototype.subtract
description: Maximum allowed duration subtracting from maximum allowed date
includes: [temporalHelpers.js]
features: [Temporal]
---*/

const max = Temporal.PlainDateTime.from({ year: 275760, month: 9, day: 13 });

const maxCases = [
  ["P547581Y4M23DT23H59M59.999999999S", "string with max years"],
  [{ years: 547581, months: 4, days: 23, nanoseconds: 86399999999999 }, "property bag with max years"],
  ["P6570976M23DT23H59M59.999999999S", "string with max months"],
  [{ months: 6570976, days: 23, nanoseconds: 86399999999999 }, "property bag with max months"],
  ["P28571428W4DT23H59M59.999999999S", "string with max weeks"],
  [{ weeks: 28_571_428, days: 4, nanoseconds: 86399999999999 }, "property bag with max weeks"],
  ["P200000000DT23H59M59.999999999S", "string with max days"],
  [{ days: 200_000_000, nanoseconds: 86399999999999 }, "property bag with max days"],
  ["PT4800000023H59M59.999999999S", "string with max hours"],
  [{ hours: 4_800_000_023, minutes: 59, seconds: 59, milliseconds: 999, microseconds: 999, nanoseconds: 999 }, "property bag with max hours"],
  ["PT288000001439M59.999999999S", "string with max minutes"],
  [{ minutes: 288_000_001_439, seconds: 59, milliseconds: 999, microseconds: 999, nanoseconds: 999 }, "property bag with max minutes"],
  ["PT17280000086399.999999999S", "string with max seconds"],
  [{ seconds: 17_280_000_086_399, nanoseconds: 999999999 }, "property bag with max seconds"],
];

for (const [arg, descr] of maxCases) {
    const result = max.subtract(arg);
    TemporalHelpers.assertPlainDateTime(result, -271821, 4, "M04", 19, 0, 0, 0, 0, 0, 1,  `operation succeeds with ${descr}`);
}

const min = Temporal.PlainDateTime.from({ year: -271821, month: 4, day: 19, nanosecond: 1 });

const minCases = [
  ["-P547581Y4M24DT23H59M59.999999999S", "string with max years"],
  [{ years: -547581, months: -4, days: -24, nanoseconds: -86399999999999 }, "property bag with max years"],
  ["-P6570976M24DT23H59M59.999999999S", "string with max months"],
  [{ months: -6570976, days: -24, nanoseconds: -86399999999999 }, "property bag with max months"],
  ["-P28571428W4DT23H59M59.999999999S", "string with max weeks"],
  [{ weeks: -28_571_428, days: -4, nanoseconds: -86399999999999 }, "property bag with max weeks"],
  ["-P200000000DT23H59M59.999999999S", "string with max days"],
  [{ days: -200_000_000, nanoseconds: -86399999999999 }, "property bag with max days"],
  ["-PT4800000023H59M59.999999999S", "string with max hours"],
  [{ hours: -4_800_000_023, minutes: -59, seconds: -59, milliseconds: -999, microseconds: -999, nanoseconds: -999 }, "property bag with max hours"],
  ["-PT288000001439M59.999999999S", "string with max minutes"],
  [{ minutes: -288_000_001_439, seconds: -59, milliseconds: -999, microseconds: -999, nanoseconds: -999 }, "property bag with max minutes"],
  ["-PT17280000086399.999999999S", "string with max seconds"],
  [{ seconds: -17_280_000_086_399, nanoseconds: -999999999 }, "property bag with max seconds"],
];

for (const [arg, descr] of minCases) {
    const result = min.subtract(arg);
    TemporalHelpers.assertPlainDateTime(result, 275760, 9, "M09", 13, 0, 0, 0, 0, 0, 0,  `operation succeeds with ${descr}`);
}
