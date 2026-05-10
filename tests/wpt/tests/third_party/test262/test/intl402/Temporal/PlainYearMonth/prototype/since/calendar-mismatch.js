// Copyright (C) 2021 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-temporal.plainyearmonth.prototype.since
description: RangeError thrown if calendars' IDs do not match
features: [Temporal]
---*/

const plainYearMonth1 = new Temporal.PlainYearMonth(2000, 1, "gregory", 1);
const plainYearMonth2 = new Temporal.PlainYearMonth(2000, 1, "japanese", 1);
assert.throws(RangeError, () => plainYearMonth1.since(plainYearMonth2));
