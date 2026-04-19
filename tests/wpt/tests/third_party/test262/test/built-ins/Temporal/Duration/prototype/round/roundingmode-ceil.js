// Copyright (C) 2022 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-temporal.duration.prototype.round
description: Tests calculations with roundingMode "ceil".
includes: [temporalHelpers.js]
features: [Temporal]
---*/

const instance = new Temporal.Duration(5, 6, 7, 8, 40, 30, 20, 123, 987, 500);
// Chosen such that 8 months forwards from relativeToForwards is the
// same number of days as 8 months backwards from relativeToBackwards
// (for convenience)
const relativeToForwards = new Temporal.PlainDate(2020, 4, 1);
const relativeToBackwards = new Temporal.PlainDate(2020, 12, 1);

const expected = [
  ["years", [6], [-5]],
  ["months", [5, 8], [-5, -7]],
  ["weeks", [5, 7, 4], [-5, -7, -3]],
  ["days", [5, 7, 0, 28], [-5, -7, 0, -27]],
  ["hours", [5, 7, 0, 27, 17], [-5, -7, 0, -27, -16]],
  ["minutes", [5, 7, 0, 27, 16, 31], [-5, -7, 0, -27, -16, -30]],
  ["seconds", [5, 7, 0, 27, 16, 30, 21], [-5, -7, 0, -27, -16, -30, -20]],
  ["milliseconds", [5, 7, 0, 27, 16, 30, 20, 124], [-5, -7, 0, -27, -16, -30, -20, -123]],
  ["microseconds", [5, 7, 0, 27, 16, 30, 20, 123, 988], [-5, -7, 0, -27, -16, -30, -20, -123, -987]],
  ["nanoseconds", [5, 7, 0, 27, 16, 30, 20, 123, 987, 500], [-5, -7, 0, -27, -16, -30, -20, -123, -987, -500]],
];

const roundingMode = "ceil";

expected.forEach(([smallestUnit, expectedPositive, expectedNegative]) => {
  const [py, pm = 0, pw = 0, pd = 0, ph = 0, pmin = 0, ps = 0, pms = 0, pµs = 0, pns = 0] = expectedPositive;
  const [ny, nm = 0, nw = 0, nd = 0, nh = 0, nmin = 0, ns = 0, nms = 0, nµs = 0, nns = 0] = expectedNegative;
  TemporalHelpers.assertDuration(
    instance.round({ smallestUnit, relativeTo: relativeToForwards, roundingMode }),
    py, pm, pw, pd, ph, pmin, ps, pms, pµs, pns,
    `rounds to ${smallestUnit} (roundingMode = ${roundingMode}, positive case)`
  );
  TemporalHelpers.assertDuration(
    instance.negated().round({ smallestUnit, relativeTo: relativeToBackwards, roundingMode }),
    ny, nm, nw, nd, nh, nmin, ns, nms, nµs, nns,
    `rounds to ${smallestUnit} (rounding mode = ${roundingMode}, negative case)`
  );
});
