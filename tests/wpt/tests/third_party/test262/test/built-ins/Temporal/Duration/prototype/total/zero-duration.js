// Copyright (C) 2025 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-temporal.duration.prototype.total
description: Totaling zero duration returns 0
features: [Temporal]
---*/

const zero = new Temporal.Duration();

let relativeToDates = [
  new Temporal.ZonedDateTime(0n, 'UTC'),
  new Temporal.PlainDateTime(1970, 1, 1)
];

let units = [ 'days', 'weeks', 'months', 'years' ];

for (const relativeTo of relativeToDates) {
  for (const unit of units) {
    assert.sameValue(zero.total({ unit, relativeTo }), 0);
  }
}
