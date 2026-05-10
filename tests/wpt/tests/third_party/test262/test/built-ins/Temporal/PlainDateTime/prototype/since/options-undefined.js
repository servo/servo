// Copyright (C) 2021 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-temporal.plaindatetime.prototype.since
description: Verify that undefined options are handled correctly.
features: [Temporal]
---*/

const earlier = new Temporal.PlainDateTime(2000, 5, 2, 12, 34, 56, 987, 654, 321);
const later = new Temporal.PlainDateTime(2000, 6, 12, 12, 34, 56, 987, 654, 322);

const explicit = later.since(earlier, undefined);
assert.sameValue(explicit.years, 0, "default largest unit is days");
assert.sameValue(explicit.months, 0, "default largest unit is days");
assert.sameValue(explicit.weeks, 0, "default largest unit is days");
assert.sameValue(explicit.days, 41, "default largest unit is days");
assert.sameValue(explicit.nanoseconds, 1, "default smallest unit is nanoseconds and no rounding");

const implicit = later.since(earlier);
assert.sameValue(implicit.years, 0, "default largest unit is days");
assert.sameValue(implicit.months, 0, "default largest unit is days");
assert.sameValue(implicit.weeks, 0, "default largest unit is days");
assert.sameValue(implicit.days, 41, "default largest unit is days");
assert.sameValue(implicit.nanoseconds, 1, "default smallest unit is nanoseconds and no rounding");

const lambda = later.since(earlier, () => {});
assert.sameValue(lambda.years, 0, "default largest unit is days");
assert.sameValue(lambda.months, 0, "default largest unit is days");
assert.sameValue(lambda.weeks, 0, "default largest unit is days");
assert.sameValue(lambda.days, 41, "default largest unit is days");
assert.sameValue(lambda.nanoseconds, 1, "default smallest unit is nanoseconds and no rounding");
