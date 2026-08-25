// Copyright (C) 2021 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-temporal.plaindatetime.prototype.until
description: Type conversions for largestUnit option
includes: [compareArray.js, temporalHelpers.js]
features: [Temporal]
---*/

const earlier = new Temporal.PlainDateTime(2000, 5, 2, 12, 34, 56, 0, 0, 0);
const later = new Temporal.PlainDateTime(2001, 6, 3, 13, 35, 57, 987, 654, 321);
TemporalHelpers.checkStringOptionWrongType("largestUnit", "year",
  (largestUnit) => earlier.until(later, { largestUnit }),
  (result, descr) => TemporalHelpers.assertDuration(result, 1, 1, 0, 1, 1, 1, 1, 987, 654, 321, descr),
);
