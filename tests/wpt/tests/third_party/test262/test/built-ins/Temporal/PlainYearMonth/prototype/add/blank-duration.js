// Copyright (C) 2025 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-temporal.plainyearmonth.prototype.add
description: Behaviour with blank duration
features: [Temporal]
includes: [temporalHelpers.js]
---*/

const ym = new Temporal.PlainYearMonth(2025, 8);
const blank = new Temporal.Duration();
const result = ym.add(blank);
TemporalHelpers.assertPlainYearMonth(result, 2025, 8, "M08", "result is unchanged");
