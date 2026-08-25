// Copyright (C) 2025 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-temporal.plainmonthday.from
description: Months and days must be non-negative integers
features: [Temporal]
---*/

assert.throws(RangeError, () => Temporal.PlainMonthDay.from({ day: 1, month: -1 }));
assert.throws(RangeError, () => Temporal.PlainMonthDay.from({ month: 1, day: -1 }));
