// Copyright (C) 2022 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-get-temporal.plainyearmonth.prototype.monthsinyear
description: monthsInYear works
features: [Temporal]
---*/

const ym = new Temporal.PlainYearMonth(1976, 11);
assert.sameValue(ym.monthsInYear, 12);
