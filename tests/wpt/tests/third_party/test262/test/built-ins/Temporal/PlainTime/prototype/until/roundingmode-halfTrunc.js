// Copyright (C) 2022 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-temporal.plaintime.prototype.until
description: Tests calculations with roundingMode "halfTrunc".
includes: [temporalHelpers.js]
features: [Temporal]
---*/

const earlier = new Temporal.PlainTime(8, 22, 36, 123, 456, 789);
const later = new Temporal.PlainTime(12, 39, 40, 987, 654, 289);

const expected = [
  ["hours", [0, 0, 0, 0, 4], [0, 0, 0, 0, -4]],
  ["minutes", [0, 0, 0, 0, 4, 17], [0, 0, 0, 0, -4, -17]],
  ["seconds", [0, 0, 0, 0, 4, 17, 5], [0, 0, 0, 0, -4, -17, -5]],
  ["milliseconds", [0, 0, 0, 0, 4, 17, 4, 864], [0, 0, 0, 0, -4, -17, -4, -864]],
  ["microseconds", [0, 0, 0, 0, 4, 17, 4, 864, 197], [0, 0, 0, 0, -4, -17, -4, -864, -197]],
  ["nanoseconds", [0, 0, 0, 0, 4, 17, 4, 864, 197, 500], [0, 0, 0, 0, -4, -17, -4, -864, -197, -500]],
];

const roundingMode = "halfTrunc";

expected.forEach(([smallestUnit, expectedPositive, expectedNegative]) => {
  const [py, pm = 0, pw = 0, pd = 0, ph = 0, pmin = 0, ps = 0, pms = 0, pµs = 0, pns = 0] = expectedPositive;
  const [ny, nm = 0, nw = 0, nd = 0, nh = 0, nmin = 0, ns = 0, nms = 0, nµs = 0, nns = 0] = expectedNegative;
  TemporalHelpers.assertDuration(
    earlier.until(later, { smallestUnit, roundingMode }),
    py, pm, pw, pd, ph, pmin, ps, pms, pµs, pns,
    `rounds to ${smallestUnit} (roundingMode = ${roundingMode}, positive case)`
  );
  TemporalHelpers.assertDuration(
    later.until(earlier, { smallestUnit, roundingMode }),
    ny, nm, nw, nd, nh, nmin, ns, nms, nµs, nns,
    `rounds to ${smallestUnit} (rounding mode = ${roundingMode}, negative case)`
  );
});
