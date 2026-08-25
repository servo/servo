// Copyright (C) 2026 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-temporal.plaindatetime.prototype.until
description: A case where date-times are internally adjusted to have the same date component
includes: [temporalHelpers.js]
features: [Temporal]
---*/

for (const largestUnit of ['years', 'months', 'weeks', 'days', 'hours']) {
  const d1 = new Temporal.PlainDateTime(2026, 1, 6, 11, 2, 0, 0, 0, 0, "gregory");
  const d2 = new Temporal.PlainDateTime(2026, 1, 7, 9, 2, 0, 0, 0, 0, "gregory");
  TemporalHelpers.assertDuration(d1.until(d2, { largestUnit }),
    0, 0, 0, 0, 22, 0, 0, 0, 0, 0, `differencing ${d1} and ${d2}`);
}
