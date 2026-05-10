// Copyright (C) 2025 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-temporal.plainyearmonth.prototype.from
description: Months must be non-negative integers
features: [Temporal]
---*/

assert.throws(RangeError, () => Temporal.PlainYearMonth.from({ year: 1, month: -1 }));
