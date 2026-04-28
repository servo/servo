// Copyright (C) 2021 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-temporal.plaintime.prototype.since
description: Verify that undefined options are handled correctly.
features: [Temporal]
---*/

const earlier = new Temporal.PlainTime(12, 34, 56, 987, 654, 321);
const later = new Temporal.PlainTime(18, 34, 56, 987, 654, 322);

const explicit = later.since(earlier, undefined);
assert.sameValue(explicit.hours, 6, "default largest unit is hours");
assert.sameValue(explicit.nanoseconds, 1, "default smallest unit is nanoseconds and no rounding");

const implicit = later.since(earlier);
assert.sameValue(implicit.hours, 6, "default largest unit is hours");
assert.sameValue(implicit.nanoseconds, 1, "default smallest unit is nanoseconds and no rounding");

const lambda = later.since(earlier, () => {});
assert.sameValue(lambda.hours, 6, "default largest unit is hours");
assert.sameValue(lambda.nanoseconds, 1, "default smallest unit is nanoseconds and no rounding");
