// Copyright (C) 2025 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-temporal.zoneddatetime.prototype.add
description: Maximum allowed duration adding to minimum allowed date
includes: [temporalHelpers.js]
features: [Temporal]
---*/

const min = Temporal.ZonedDateTime.from({ year: -271821, month: 4, day: 20, timeZone: "UTC" });

const maxCases = [
  ["P547581Y4M24D", "string with max years"],
  [{ years: 547581, months: 4, days: 24 }, "property bag with max years"],
  ["P6570976M24D", "string with max months"],
  [{ months: 6570976, days: 24 }, "property bag with max months"],
  ["P28571428W4D", "string with max weeks"],
  [{ weeks: 28_571_428, days: 4 }, "property bag with max weeks"],
  ["P200000000D", "string with max days"],
  [{ days: 200_000_000 }, "property bag with max days"],
  ["PT4800000000H", "string with max hours"],
  [{ hours: 4_800_000_000 }, "property bag with max hours"],
  ["PT288000000000M", "string with max minutes"],
  [{ minutes: 288_000_000_000 }, "property bag with max minutes"],
  ["PT17280000000000S", "string with max seconds"],
  [{ seconds: 17_280_000_000_000 }, "property bag with max seconds"],
];

for (const [arg, descr] of maxCases) {
    const result = min.add(arg);
    TemporalHelpers.assertPlainDateTime(result.toPlainDateTime(), 275760, 9, "M09", 13, 0, 0, 0, 0, 0, 0,  `operation succeeds with ${descr}`)
}

const max = Temporal.ZonedDateTime.from({ year: 275760, month: 9, day: 13, timeZone: "UTC" });

const minCases = [
  ["-P547581Y4M23D", "string with max years"],
  [{ years: -547581, months: -4, days: -23 }, "property bag with max years"],
  ["-P6570976M23D", "string with max months"],
  [{ months: -6570976, days: -23 }, "property bag with max months"],
  ["-P28571428W4D", "string with max weeks"],
  [{ weeks: -28_571_428, days: -4 }, "property bag with max weeks"],
  ["-P200000000D", "string with max days"],
  [{ days: -200_000_000 }, "property bag with max days"],
  ["-PT4800000000H", "string with max hours"],
  [{ hours: -4_800_000_000 }, "property bag with max hours"],
  ["-PT288000000000M", "string with max minutes"],
  [{ minutes: -288_000_000_000 }, "property bag with max minutes"],
  ["-PT17280000000000S", "string with max seconds"],
  [{ seconds: -17_280_000_000_000 }, "property bag with max seconds"],
];

for (const [arg, descr] of minCases) {
    const result = max.add(arg);
    TemporalHelpers.assertPlainDateTime(result.toPlainDateTime(), -271821, 4, "M04", 20, 0, 0, 0, 0, 0, 0,  `operation succeeds with ${descr}`);
}
