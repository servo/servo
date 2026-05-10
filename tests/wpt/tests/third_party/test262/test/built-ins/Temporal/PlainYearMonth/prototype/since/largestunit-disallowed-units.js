// Copyright (C) 2021 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-temporal.plainyearmonth.prototype.since
description: Since throws on to0-small largestUnit
features: [Temporal, arrow-function]
---*/

const earlier = new Temporal.PlainYearMonth(2019, 1);
const later = new Temporal.PlainYearMonth(2021, 9);

[
  'weeks',
  'days',
  'hours',
  'minutes',
  'seconds',
  'milliseconds',
  'microseconds',
  'nanoseconds'
].forEach((largestUnit) => {
  assert.throws(RangeError, () => later.since(earlier, { largestUnit }),`throws on disallowed or invalid largestUnit: ${largestUnit}`);
});
