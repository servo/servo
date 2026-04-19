// Copyright (C) 2025 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-temporal.duration.prototype.round
description: Behaviour with blank duration
features: [Temporal]
includes: [temporalHelpers.js]
---*/

const blank = new Temporal.Duration();
const plainRelativeTo = new Temporal.PlainDate(2025, 8, 22);
const zonedRelativeTo = new Temporal.ZonedDateTime(1n, "UTC");

for (const smallestUnit of ['days', 'hours', 'minutes', 'seconds', 'milliseconds', 'microseconds', 'nanoseconds']) {
  let result = blank.round(smallestUnit);
  TemporalHelpers.assertDuration(result, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, `round to ${smallestUnit} without relativeTo`);

  result = blank.round({ smallestUnit, relativeTo: plainRelativeTo });
  TemporalHelpers.assertDuration(result, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, `round to ${smallestUnit} with PlainDate relativeTo`);

  result = blank.round({ smallestUnit, relativeTo: zonedRelativeTo });
  TemporalHelpers.assertDuration(result, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, `round to ${smallestUnit} with ZonedDateTime relativeTo`);
}

for (const smallestUnit of ['years', 'months', 'weeks']) {
  let result = blank.round({ smallestUnit, relativeTo: plainRelativeTo });
  TemporalHelpers.assertDuration(result, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, `round to ${smallestUnit} with PlainDate relativeTo`);

  result = blank.round({ smallestUnit, relativeTo: zonedRelativeTo });
  TemporalHelpers.assertDuration(result, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, `round to ${smallestUnit} with ZonedDateTime relativeTo`);
}
