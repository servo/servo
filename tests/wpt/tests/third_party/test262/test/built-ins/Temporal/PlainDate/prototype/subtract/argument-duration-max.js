// Copyright (C) 2023 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-temporal.plaindate.prototype.subtract
description: Maximum allowed duration
includes: [temporalHelpers.js]
features: [Temporal]
---*/

const instance = new Temporal.PlainDate(1970, 1, 1);

const maxCases = [
  ["P273790Y8M12DT23H59M59.999999999S", "string with max years"],
  [{ years: 273790, months: 8, days: 12, nanoseconds: 86399999999999 }, "property bag with max years"],
  ["P3285488M12DT23H59M59.999999999S", "string with max months"],
  [{ months: 3285488, days: 12, nanoseconds: 86399999999999 }, "property bag with max months"],
  ["P14285714W3DT23H59M59.999999999S", "string with max weeks"],
  [{ weeks: 14285714, days: 3, nanoseconds: 86399999999999 }, "property bag with max weeks"],
  ["P100000001DT23H59M59.999999999S", "string with max days"],
  [{ days: 100000001, nanoseconds: 86399999999999 }, "property bag with max days"],
  ["PT2400000047H59M59.999999999S", "string with max hours"],
  [{ hours: 2400000047, nanoseconds: 3599999999999 }, "property bag with max hours"],
  ["PT144000002879M59.999999999S", "string with max minutes"],
  [{ minutes: 144000002879, nanoseconds: 59999999999 }, "property bag with max minutes"],
  ["PT8640000172799.999999999S", "string with max seconds"],
  [{ seconds: 8640000172799, nanoseconds: 999999999 }, "property bag with max seconds"],
];

for (const [arg, descr] of maxCases) {
  const result = instance.subtract(arg);
  TemporalHelpers.assertPlainDate(result, -271821, 4, "M04", 19, `operation succeeds with ${descr}`);
}

const minCases = [
  ["-P273790Y8M12DT23H59M59.999999999S", "string with min years"],
  [{ years: -273790, months: -8, days: -12, nanoseconds: -86399999999999 }, "property bag with min years"],
  ["-P3285488M12DT23H59M59.999999999S", "string with min months"],
  [{ months: -3285488, days: -12, nanoseconds: -86399999999999 }, "property bag with min months"],
  ["-P14285714W2DT23H59M59.999999999S", "string with min weeks"],
  [{ weeks: -14285714, days: -2, nanoseconds: -86399999999999 }, "property bag with min weeks"],
  ["-P100000000DT23H59M59.999999999S", "string with min days"],
  [{ days: -100000000, nanoseconds: -86399999999999 }, "property bag with min days"],
  ["-PT2400000023H59M59.999999999S", "string with min hours"],
  [{ hours: -2400000023, nanoseconds: -3599999999999 }, "property bag with min hours"],
  ["-PT144000001439M59.999999999S", "string with min minutes"],
  [{ minutes: -144000001439, nanoseconds: -59999999999 }, "property bag with min minutes"],
  ["-PT8640000086399.999999999S", "string with min seconds"],
  [{ seconds: -8640000086399, nanoseconds: -999999999 }, "property bag with min seconds"],
];

for (const [arg, descr] of minCases) {
  const result = instance.subtract(arg);
  TemporalHelpers.assertPlainDate(result, 275760, 9, "M09", 13, `operation succeeds with ${descr}`);
}
