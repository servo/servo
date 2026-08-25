// Copyright (C) 2021 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-temporal.plainyearmonth.prototype.with
description: Verify that undefined options are handled correctly.
features: [Temporal]
---*/

const yearmonth = new Temporal.PlainYearMonth(2000, 2);
const fields = { month: 13 };

const explicit = yearmonth.with(fields, undefined);
assert.sameValue(explicit.month, 12, "default overflow is constrain");

const implicit = yearmonth.with(fields);
assert.sameValue(implicit.month, 12, "default overflow is constrain");

const lambda = yearmonth.with(fields, () => {});
assert.sameValue(lambda.month, 12, "default overflow is constrain");
