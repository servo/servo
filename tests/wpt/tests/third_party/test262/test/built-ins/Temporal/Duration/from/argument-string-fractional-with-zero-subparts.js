// Copyright (C) 2022 André Bargull. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-temporal.duration.from
description: >
  Throws when a fractional unit is present and a sub-part is zero.
features: [Temporal]
---*/

const invalid = [
  // Hours fraction with whole minutes.
  "PT0.1H0M",

  // Hours fraction with fractional minutes.
  "PT0.1H0.0M",

  // Hours fraction with whole seconds.
  "PT0.1H0S",

  // Hours fraction with fractional seconds.
  "PT0.1H0.0S",

  // Minutes fraction with whole seconds.
  "PT0.1M0S",

  // Minutes fraction with fractional seconds.
  "PT0.1M0.0S",
];

for (let string of invalid) {
  assert.throws(RangeError, () => Temporal.Duration.from(string));
}
