// Copyright (C) 2022 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-temporal.zoneddatetime.prototype.until
description: Tests calculations with roundingMode "ceil".
includes: [temporalHelpers.js]
features: [Temporal]
---*/

const earlier = new Temporal.ZonedDateTime(1546935756_123_456_789n /* 2019-01-08T08:22:36.123456789+00:00 */, "UTC");
const later = new Temporal.ZonedDateTime(1631018380_987_654_289n /* 2021-09-07T12:39:40.987654289+00:00 */, "UTC");

const expected = [
  ["years", [3], [-2]],
  ["months", [0, 32], [0, -31]],
  ["weeks", [0, 0, 140], [0, 0, -139]],
  ["days", [0, 0, 0, 974], [0, 0, 0, -973]],
  ["hours", [0, 0, 0, 0, 23357], [0, 0, 0, 0, -23356]],
  ["minutes", [0, 0, 0, 0, 23356, 18], [0, 0, 0, 0, -23356, -17]],
  ["seconds", [0, 0, 0, 0, 23356, 17, 5], [0, 0, 0, 0, -23356, -17, -4]],
  ["milliseconds", [0, 0, 0, 0, 23356, 17, 4, 865], [0, 0, 0, 0, -23356, -17, -4, -864]],
  ["microseconds", [0, 0, 0, 0, 23356, 17, 4, 864, 198], [0, 0, 0, 0, -23356, -17, -4, -864, -197]],
  ["nanoseconds", [0, 0, 0, 0, 23356, 17, 4, 864, 197, 500], [0, 0, 0, 0, -23356, -17, -4, -864, -197, -500]],
];

const roundingMode = "ceil";

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
