// Copyright (C) 2021 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-temporal.plaindate.prototype.subtract
description: Verify that undefined options are handled correctly.
features: [Temporal]
---*/

const date = new Temporal.PlainDate(2000, 3, 31);
const duration = { months: 1 };

const explicit = date.subtract(duration, undefined);
assert.sameValue(explicit.month, 2, "default overflow is constrain");
assert.sameValue(explicit.day, 29, "default overflow is constrain");

const implicit = date.subtract(duration);
assert.sameValue(implicit.month, 2, "default overflow is constrain");
assert.sameValue(implicit.day, 29, "default overflow is constrain");

const lambda = date.subtract(duration, () => {});
assert.sameValue(lambda.month, 2, "default overflow is constrain");
assert.sameValue(lambda.day, 29, "default overflow is constrain");
