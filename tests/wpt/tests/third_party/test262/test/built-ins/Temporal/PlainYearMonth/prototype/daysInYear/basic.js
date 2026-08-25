// Copyright (C) 2022 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-get-temporal.plainyearmonth.prototype.daysinyear
description: daysInYear works
features: [Temporal]
---*/

assert.sameValue((new Temporal.PlainYearMonth(1976, 11)).daysInYear, 366, "leap year");
assert.sameValue((new Temporal.PlainYearMonth(1977, 11)).daysInYear, 365, "non-leap year");
