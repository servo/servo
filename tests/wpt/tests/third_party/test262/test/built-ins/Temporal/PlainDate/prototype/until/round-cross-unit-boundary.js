// Copyright (C) 2023 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-temporal.plaindate.prototype.until
description: Rounding can cross unit boundaries up to largestUnit
includes: [temporalHelpers.js]
features: [Temporal]
---*/

const earlier = new Temporal.PlainDate(2022, 1, 1);
const later = new Temporal.PlainDate(2023, 12, 25);
const duration = earlier.until(later, { largestUnit: "years", smallestUnit: "months", roundingMode: "expand" });
TemporalHelpers.assertDuration(duration, 2, 0, 0, 0, 0, 0, 0, 0, 0, 0, "1 year 11 months balances to 2 years");
