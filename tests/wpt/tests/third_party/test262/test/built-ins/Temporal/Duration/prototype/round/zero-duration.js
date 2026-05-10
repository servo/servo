// Copyright (C) 2025 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-temporal.duration.prototype.round
description: Rounding zero duration returns 0
includes: [temporalHelpers.js]
features: [Temporal]
---*/

const zero = new Temporal.Duration();

let relativeToDates = [
  new Temporal.ZonedDateTime(0n, 'UTC'),
  new Temporal.PlainDateTime(1970, 1, 1)
];

let units = [
  { smallestUnit: 'hours', largestUnit: 'days' },
  { smallestUnit: 'minutes', largestUnit: 'days' },
  { smallestUnit: 'seconds', largestUnit: 'days' },
  { smallestUnit: 'hours', largestUnit: 'weeks' },
  { smallestUnit: 'minutes', largestUnit: 'weeks' },
  { smallestUnit: 'seconds', largestUnit: 'weeks' },
  { smallestUnit: 'hours', largestUnit: 'months' },
  { smallestUnit: 'minutes', largestUnit: 'months' },
  { smallestUnit: 'seconds', largestUnit: 'months' },
  { smallestUnit: 'hours', largestUnit: 'years' },
  { smallestUnit: 'minutes', largestUnit: 'years' },
  { smallestUnit: 'seconds', largestUnit: 'years' }
];

for (const relativeTo of relativeToDates) {
  for (const unit of units) {
    TemporalHelpers.assertDuration(zero.round({
      smallestUnit: unit.smallestUnit,
      largestUnit: unit.largestUnit,
      relativeTo
    }),
    0, 0, 0, 0, 0, 0, 0, 0, 0, 0);
  }
}
