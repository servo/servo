// Copyright (C) 2021 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-temporal.plainyearmonth.from
description: Verify that undefined options are handled correctly.
features: [Temporal]
---*/

const fields = { year: 2000, month: 13 };

const explicit = Temporal.PlainYearMonth.from(fields, undefined);
assert.sameValue(explicit.month, 12, "default overflow is constrain");

const implicit = Temporal.PlainYearMonth.from(fields);
assert.sameValue(implicit.month, 12, "default overflow is constrain");

const lambda = Temporal.PlainYearMonth.from(fields, () => {});
assert.sameValue(lambda.month, 12, "default overflow is constrain");
