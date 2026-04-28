// Copyright (C) 2021 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-temporal.instant.prototype.until
description: Verify that undefined options are handled correctly.
features: [BigInt, Temporal]
---*/

const earlier = new Temporal.Instant(957270896_987_654_321n);
const later = new Temporal.Instant(959949296_987_654_322n);

const explicit = earlier.until(later, undefined);
assert.sameValue(explicit.years, 0, "default largest unit is seconds");
assert.sameValue(explicit.months, 0, "default largest unit is seconds");
assert.sameValue(explicit.weeks, 0, "default largest unit is seconds");
assert.sameValue(explicit.days, 0, "default largest unit is seconds");
assert.sameValue(explicit.hours, 0, "default largest unit is seconds");
assert.sameValue(explicit.minutes, 0, "default largest unit is seconds");
assert.sameValue(explicit.seconds, 2678400, "default largest unit is seconds");
assert.sameValue(explicit.nanoseconds, 1, "default smallest unit is nanoseconds and no rounding");

const implicit = earlier.until(later);
assert.sameValue(implicit.years, 0, "default largest unit is seconds");
assert.sameValue(implicit.months, 0, "default largest unit is seconds");
assert.sameValue(implicit.weeks, 0, "default largest unit is seconds");
assert.sameValue(implicit.days, 0, "default largest unit is seconds");
assert.sameValue(implicit.hours, 0, "default largest unit is seconds");
assert.sameValue(implicit.minutes, 0, "default largest unit is seconds");
assert.sameValue(implicit.seconds, 2678400, "default largest unit is seconds");
assert.sameValue(implicit.nanoseconds, 1, "default smallest unit is nanoseconds and no rounding");

const lambda = earlier.until(later, () => {});
assert.sameValue(lambda.years, 0, "default largest unit is seconds");
assert.sameValue(lambda.months, 0, "default largest unit is seconds");
assert.sameValue(lambda.weeks, 0, "default largest unit is seconds");
assert.sameValue(lambda.days, 0, "default largest unit is seconds");
assert.sameValue(lambda.hours, 0, "default largest unit is seconds");
assert.sameValue(lambda.minutes, 0, "default largest unit is seconds");
assert.sameValue(lambda.seconds, 2678400, "default largest unit is seconds");
assert.sameValue(lambda.nanoseconds, 1, "default smallest unit is nanoseconds and no rounding");
