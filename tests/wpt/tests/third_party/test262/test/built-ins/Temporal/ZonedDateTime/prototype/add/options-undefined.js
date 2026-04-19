// Copyright (C) 2021 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-temporal.zoneddatetime.prototype.add
description: Verify that undefined options are handled correctly.
features: [BigInt, Temporal]
---*/

const datetime = new Temporal.ZonedDateTime(949322096_987_654_321n, "UTC");
const duration = { months: 1 };

const explicit = datetime.add(duration, undefined);
assert.sameValue(explicit.month, 2, "default overflow is constrain");
assert.sameValue(explicit.day, 29, "default overflow is constrain");

const implicit = datetime.add(duration);
assert.sameValue(implicit.month, 2, "default overflow is constrain");
assert.sameValue(implicit.day, 29, "default overflow is constrain");

const lambda = datetime.add(duration, () => {});
assert.sameValue(lambda.month, 2, "default overflow is constrain");
assert.sameValue(lambda.day, 29, "default overflow is constrain");
