// Copyright (C) 2025 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-temporal.duration.prototype.total
description: Behaviour with blank duration
features: [Temporal]
---*/

const blank = new Temporal.Duration();
const plainRelativeTo = new Temporal.PlainDate(2025, 8, 22);
const zonedRelativeTo = new Temporal.ZonedDateTime(1n, "UTC");

for (const unit of ['days', 'hours', 'minutes', 'seconds', 'milliseconds', 'microseconds', 'nanoseconds']) {
  let result = blank.total(unit);
  assert.sameValue(result, 0, `total of ${unit} without relativeTo`);

  result = blank.total({ unit, relativeTo: plainRelativeTo });
  assert.sameValue(result, 0, `total of ${unit} with PlainDate relativeTo`);

  result = blank.total({ unit, relativeTo: zonedRelativeTo });
  assert.sameValue(result, 0, `total of ${unit} with ZonedDateTime relativeTo`);
}

for (const unit of ['years', 'months', 'weeks']) {
  let result = blank.total({ unit, relativeTo: plainRelativeTo });
  assert.sameValue(result, 0, `total of ${unit} with PlainDate relativeTo`);

  result = blank.total({ unit, relativeTo: zonedRelativeTo });
  assert.sameValue(result, 0, `total of ${unit} with ZonedDateTime relativeTo`);
}
