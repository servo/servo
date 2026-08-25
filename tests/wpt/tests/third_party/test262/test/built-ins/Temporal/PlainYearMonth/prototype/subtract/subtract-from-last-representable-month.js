// Copyright (C) 2024 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-temporal.plainyearmonth.prototype.subtract
description: Subtracting duration from last representable month works
features: [Temporal]
includes: [temporalHelpers.js]
---*/

const lastMonth = new Temporal.PlainYearMonth(275760, 9);

TemporalHelpers.assertPlainYearMonth(lastMonth.subtract({ months: 1 }), 275760, 8, "M08", "1 month");
TemporalHelpers.assertPlainYearMonth(lastMonth.subtract({ years: 1 }), 275759, 9, "M09", "1 year");

