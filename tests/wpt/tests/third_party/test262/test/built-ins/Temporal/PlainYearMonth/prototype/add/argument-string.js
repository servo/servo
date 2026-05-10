// Copyright (C) 2020 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-temporal.plainyearmonth.prototype.add
description: A string is parsed into the correct object when passed as the argument
includes: [temporalHelpers.js]
features: [Temporal]
---*/

const instance = Temporal.PlainYearMonth.from({ year: 2000, month: 5 });
const result = instance.add("P3M");
TemporalHelpers.assertPlainYearMonth(result, 2000, 8, "M08");
