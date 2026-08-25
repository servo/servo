// Copyright (C) 2022 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-temporal.plaindate.prototype.until
description: Tests calculations with roundingMode "ceil".
includes: [temporalHelpers.js]
features: [Temporal]
---*/

const earlier = new Temporal.PlainDate(2019, 1, 8);
const later = new Temporal.PlainDate(2021, 9, 7);

const expected = [
  ["years", [3], [-2]],
  ["months", [0, 32], [0, -31]],
  ["weeks", [0, 0, 139], [0, 0, -139]],
  ["days", [0, 0, 0, 973], [0, 0, 0, -973]],
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
