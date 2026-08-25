// Copyright (C) 2022 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-temporal.plainyearmonth.from
description: RangeError thrown if a string with trailing junk is used as a PlainYearMonth
features: [Temporal]
---*/

assert.throws(RangeError, () => Temporal.PlainYearMonth.from("1976-11junk"));
