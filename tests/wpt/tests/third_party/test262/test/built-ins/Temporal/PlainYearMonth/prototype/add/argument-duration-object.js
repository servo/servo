// Copyright (C) 2022 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-temporal.plainyearmonth.prototype.add
description: A Duration object is supported as the argument
includes: [temporalHelpers.js]
features: [Temporal]
---*/

const nov94 = Temporal.PlainYearMonth.from("1994-11");
const diff = Temporal.Duration.from("P18Y7M");
TemporalHelpers.assertPlainYearMonth(nov94.add(diff), 2013, 6, "M06");
