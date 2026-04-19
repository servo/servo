// Copyright (C) 2022 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-temporal.plaindatetime.prototype.since
description: Tests calculations with roundingMode "ceil".
includes: [temporalHelpers.js]
features: [Temporal]
---*/

const earlier = new Temporal.PlainDateTime(2019, 1, 8, 8, 22, 36, 123, 456, 789);
const later = new Temporal.PlainDateTime(2021, 9, 7, 12, 39, 40, 987, 654, 289);

const expected = [
  ["years", [3], [-2]],
  ["months", [0, 32], [0, -31]],
  ["weeks", [0, 0, 140], [0, 0, -139]],
  ["days", [0, 0, 0, 974], [0, 0, 0, -973]],
  ["hours", [0, 0, 0, 973, 5], [0, 0, 0, -973, -4]],
  ["minutes", [0, 0, 0, 973, 4, 18], [0, 0, 0, -973, -4, -17]],
  ["seconds", [0, 0, 0, 973, 4, 17, 5], [0, 0, 0, -973, -4, -17, -4]],
  ["milliseconds", [0, 0, 0, 973, 4, 17, 4, 865], [0, 0, 0, -973, -4, -17, -4, -864]],
  ["microseconds", [0, 0, 0, 973, 4, 17, 4, 864, 198], [0, 0, 0, -973, -4, -17, -4, -864, -197]],
  ["nanoseconds", [0, 0, 0, 973, 4, 17, 4, 864, 197, 500], [0, 0, 0, -973, -4, -17, -4, -864, -197, -500]],
];

const roundingMode = "ceil";

expected.forEach(([smallestUnit, expectedPositive, expectedNegative]) => {
  const [py, pm = 0, pw = 0, pd = 0, ph = 0, pmin = 0, ps = 0, pms = 0, pµs = 0, pns = 0] = expectedPositive;
  const [ny, nm = 0, nw = 0, nd = 0, nh = 0, nmin = 0, ns = 0, nms = 0, nµs = 0, nns = 0] = expectedNegative;
  TemporalHelpers.assertDuration(
    later.since(earlier, { smallestUnit, roundingMode }),
    py, pm, pw, pd, ph, pmin, ps, pms, pµs, pns,
    `rounds to ${smallestUnit} (roundingMode = ${roundingMode}, positive case)`
  );
  TemporalHelpers.assertDuration(
    earlier.since(later, { smallestUnit, roundingMode }),
    ny, nm, nw, nd, nh, nmin, ns, nms, nµs, nns,
    `rounds to ${smallestUnit} (rounding mode = ${roundingMode}, negative case)`
  );
});
