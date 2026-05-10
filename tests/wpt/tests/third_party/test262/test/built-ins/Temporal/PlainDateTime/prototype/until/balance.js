// Copyright (C) 2021 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-temporal.plaindatetime.prototype.until
description: Results with opposite-sign components (e.g. months=1, hours=-1) are balanced correctly
includes: [temporalHelpers.js]
features: [Temporal]
---*/

const a = Temporal.PlainDateTime.from("2017-10-05T08:07:14+00:00[UTC]");
const b = Temporal.PlainDateTime.from("2021-03-05T03:32:45+00:00[UTC]");
const c = Temporal.PlainDateTime.from("2021-03-05T09:32:45+00:00[UTC]");

const r1 = a.until(b, { largestUnit: "months" });
TemporalHelpers.assertDuration(r1, 0, 40, 0, 27, 19, 25, 31, 0, 0, 0, "r1");
assert.sameValue(a.add(r1).toString(), b.toString(), "a.add(r1)");

const r2 = b.until(a, { largestUnit: "months" });
TemporalHelpers.assertDuration(r2, 0, -40, 0, -30, -19, -25, -31, 0, 0, 0, "r2");
assert.sameValue(b.add(r2).toString(), a.toString(), "b.add(r2)");

const r3 = c.until(a, { largestUnit: "months" });
TemporalHelpers.assertDuration(r3, 0, -41, 0, 0, -1, -25, -31, 0, 0, 0, "r3");
assert.sameValue(c.add(r3).toString(), a.toString(), "c.add(r3)");
