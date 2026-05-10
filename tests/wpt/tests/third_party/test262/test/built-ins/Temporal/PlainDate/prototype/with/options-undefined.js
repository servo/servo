// Copyright (C) 2021 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-temporal.plaindate.prototype.with
description: Verify that undefined options are handled correctly.
features: [Temporal]
---*/

const date = new Temporal.PlainDate(2000, 2, 2);
const fields = { day: 31 };

const explicit = date.with(fields, undefined);
assert.sameValue(explicit.month, 2, "default overflow is constrain");
assert.sameValue(explicit.day, 29, "default overflow is constrain");

const implicit = date.with(fields);
assert.sameValue(implicit.month, 2, "default overflow is constrain");
assert.sameValue(implicit.day, 29, "default overflow is constrain");

const lambda = date.with(fields, () => {});
assert.sameValue(lambda.month, 2, "default overflow is constrain");
assert.sameValue(lambda.day, 29, "default overflow is constrain");
