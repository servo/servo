// Copyright (C) 2022 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-temporal.plainyearmonth.prototype.subtract
description: A Duration object is supported as the argument
includes: [temporalHelpers.js]
features: [Temporal]
---*/

const jun13 = Temporal.PlainYearMonth.from("2013-06");
const diff = Temporal.Duration.from("P18Y7M");
TemporalHelpers.assertPlainYearMonth(jun13.subtract(diff), 1994, 11, "M11");
