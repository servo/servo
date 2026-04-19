// Copyright (C) 2021 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-temporal.plaindatetime.from
description: Verify that undefined options are handled correctly.
features: [Temporal]
---*/

const fields = { year: 2000, month: 13, day: 2 };

const explicit = Temporal.PlainDateTime.from(fields, undefined);
assert.sameValue(explicit.month, 12, "default overflow is constrain");

const implicit = Temporal.PlainDateTime.from(fields);
assert.sameValue(implicit.month, 12, "default overflow is constrain");

const lambda = Temporal.PlainDateTime.from(fields, () => {});
assert.sameValue(lambda.month, 12, "default overflow is constrain");
