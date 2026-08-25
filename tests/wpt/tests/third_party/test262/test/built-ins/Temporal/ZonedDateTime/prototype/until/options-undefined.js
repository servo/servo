// Copyright (C) 2021 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-temporal.zoneddatetime.prototype.until
description: Verify that undefined options are handled correctly.
features: [BigInt, Temporal]
---*/

const earlier = new Temporal.ZonedDateTime(957270896_987_654_321n, "UTC");
const later = new Temporal.ZonedDateTime(959949296_987_654_322n, "UTC");

const explicit = earlier.until(later, undefined);
assert.sameValue(explicit.years, 0, "default largest unit is hours");
assert.sameValue(explicit.months, 0, "default largest unit is hours");
assert.sameValue(explicit.weeks, 0, "default largest unit is hours");
assert.sameValue(explicit.days, 0, "default largest unit is hours");
assert.sameValue(explicit.hours, 744, "default largest unit is hours");
assert.sameValue(explicit.nanoseconds, 1, "default smallest unit is nanoseconds and no rounding");

const implicit = earlier.until(later);
assert.sameValue(implicit.years, 0, "default largest unit is hours");
assert.sameValue(implicit.months, 0, "default largest unit is hours");
assert.sameValue(implicit.weeks, 0, "default largest unit is hours");
assert.sameValue(implicit.days, 0, "default largest unit is hours");
assert.sameValue(implicit.hours, 744, "default largest unit is hours");
assert.sameValue(implicit.nanoseconds, 1, "default smallest unit is nanoseconds and no rounding");

const lambda = earlier.until(later, () => {});
assert.sameValue(lambda.years, 0, "default largest unit is hours");
assert.sameValue(lambda.months, 0, "default largest unit is hours");
assert.sameValue(lambda.weeks, 0, "default largest unit is hours");
assert.sameValue(lambda.days, 0, "default largest unit is hours");
assert.sameValue(lambda.hours, 744, "default largest unit is hours");
assert.sameValue(lambda.nanoseconds, 1, "default smallest unit is nanoseconds and no rounding");
