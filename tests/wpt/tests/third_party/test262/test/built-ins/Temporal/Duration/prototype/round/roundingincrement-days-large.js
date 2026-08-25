// Copyright (C) 2026 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-temporal.duration.prototype.round
description: Calculation with a large, but not too large, roundingIncrement
features: [Temporal]
includes: [temporalHelpers.js]
---*/

const plainRelativeTo = new Temporal.PlainDate(1970, 1, 1);
const zonedRelativeTo = new Temporal.ZonedDateTime(0n, "UTC");

const relativeToTests = [
  [undefined, 'no'],
  [plainRelativeTo, 'plain'],
  [zonedRelativeTo, 'zoned'],
];

const duration1 = new Temporal.Duration(0, 0, 0, 0, 0, 0, 9007199254, 740, 991, 0);
for (const [relativeTo, descr] of relativeToTests) {
  const result = duration1.round({ smallestUnit: 'days', roundingIncrement: 1e7, relativeTo });
  TemporalHelpers.assertDuration(result, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, `round to 1e7 days with ${descr} relativeTo`);
}

const duration2 = new Temporal.Duration(0, 0, 0, 1);
for (const [relativeTo, descr] of relativeToTests) {
  const result = duration2.round({ smallestUnit: 'days', roundingIncrement: 1e8 - 1, roundingMode: 'ceil', relativeTo });
  TemporalHelpers.assertDuration(result, 0, 0, 0, 99999999, 0, 0, 0, 0, 0, 0, `round to 1e8-1 days with ${descr} relativeTo`);
}
