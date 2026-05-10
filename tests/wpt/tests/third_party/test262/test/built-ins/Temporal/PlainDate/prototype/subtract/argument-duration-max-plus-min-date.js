// Copyright (C) 2025 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-temporal.plaindate.prototype.subtract
description: Maximum allowed duration subtracting from maximum allowed date
includes: [temporalHelpers.js]
features: [Temporal]
---*/

const max = Temporal.PlainDate.from({ year: 275760, month: 9, day: 13 });

const maxCases = [
  ["P547581Y4M24DT23H59M59.999999999S", "string with max years"],
  [{ years: 547581, months: 4, days: 24, nanoseconds: 86399999999999 }, "property bag with max years"],
  ["P6570976M24DT23H59M59.999999999S", "string with max months"],
  [{ months: 6570976, days: 24, nanoseconds: 86399999999999 }, "property bag with max months"],
  ["P28571428W5DT23H59M59.999999999S", "string with max weeks"],
  [{ weeks: 28_571_428, days: 5, nanoseconds: 86399999999999 }, "property bag with max weeks"],
  ["P200000001DT23H59M59.999999999S", "string with max days"],
  [{ days: 200_000_001, nanoseconds: 86399999999999 }, "property bag with max days"],
  ["PT4800000047H59M59.999999999S", "string with max hours"],
  [{ hours: 4_800_000_047, minutes: 59, seconds: 59, milliseconds: 999, microseconds: 999, nanoseconds: 999 }, "property bag with max hours"],
  ["PT288000002879M59.999999999S", "string with max minutes"],
  [{ minutes: 288_000_002_879, seconds: 59, milliseconds: 999, microseconds: 999, nanoseconds: 999 }, "property bag with max minutes"],
  ["PT17280000172799.999999998S", "string with max seconds"],
  [{ seconds: 17_280_000_172_799, nanoseconds: 999999998 }, "property bag with max seconds"],
];

for (const [arg, descr] of maxCases) {
    const result = max.subtract(arg);
    TemporalHelpers.assertPlainDate(result, -271821, 4, "M04", 19,  `operation succeeds with ${descr}`);
}

const min = Temporal.PlainDate.from({ year: -271821, month: 4, day: 19 });

const minCases = [
  ["-P547581Y4M25DT23H59M59.999999999S", "string with max years"],
  [{ years: -547581, months: -4, days: -25, nanoseconds: -86399999999999 }, "property bag with max years"],
  ["-P6570976M25DT23H59M59.999999999S", "string with max months"],
  [{ months: -6570976, days: -25, nanoseconds: -86399999999999 }, "property bag with max months"],
  ["-P28571428W5DT23H59M59.999999999S", "string with max weeks"],
  [{ weeks: -28_571_428, days: -5, nanoseconds: -86399999999999 }, "property bag with max weeks"],
  ["-P200000001DT23H59M59.999999999S", "string with max days"],
  [{ days: -200_000_001, nanoseconds: -86399999999999 }, "property bag with max days"],
  ["-PT4800000047H59M59.999999999S", "string with max hours"],
  [{ hours: -4_800_000_047, minutes: -59, seconds: -59, milliseconds: -999, microseconds: -999, nanoseconds: -999 }, "property bag with max hours"],
  ["-PT288000002879M59.999999999S", "string with max minutes"],
  [{ minutes: -288_000_002_879, seconds: -59, milliseconds: -999, microseconds: -999, nanoseconds: -999 }, "property bag with max minutes"],
  ["-PT17280000172799.999999998S", "string with max seconds"],
  [{ seconds: -17_280_000_172_799, nanoseconds: -999999998 }, "property bag with max seconds"],
];

for (const [arg, descr] of minCases) {
    const result = min.subtract(arg);
    TemporalHelpers.assertPlainDate(result, 275760, 9, "M09", 13,  `operation succeeds with ${descr}`);
}
