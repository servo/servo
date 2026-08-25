// Copyright (C) 2021 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-temporal.plainyearmonth.prototype.since
description: Type conversions for largestUnit option
includes: [compareArray.js, temporalHelpers.js]
features: [Temporal]
---*/

const earlier = new Temporal.PlainYearMonth(2000, 5);
const later = new Temporal.PlainYearMonth(2001, 6);
TemporalHelpers.checkStringOptionWrongType("largestUnit", "month",
  (largestUnit) => later.since(earlier, { largestUnit }),
  (result, descr) => TemporalHelpers.assertDuration(result, 0, 13, 0, 0, 0, 0, 0, 0, 0, 0, descr),
);
