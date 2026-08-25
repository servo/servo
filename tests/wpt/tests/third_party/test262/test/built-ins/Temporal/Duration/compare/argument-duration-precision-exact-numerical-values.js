// Copyright (C) 2023 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-temporal.duration.from
description: >
    Duration-like argument performs the range check with minimal floating point
    precision loss
features: [Temporal]
---*/

// Based on a test case by André Bargull

const cases = [
  [
    {
      milliseconds: 4503599627370497_000,  // ℝ(𝔽(4503599627370497000)) = 4503599627370497024
      microseconds: 4503599627370495_000000,  // ℝ(𝔽(4503599627370495000000)) = 4503599627370494951424
    },
    // 4503599627370497024 / 1000 + 4503599627370494951424 / 1000000 is
    // 9007199254740991.975424, which is below the limit of 2**53
    "case where floating point inaccuracy brings total below limit, positive"
  ],
  [
    {
      milliseconds: -4503599627370497_000,
      microseconds: -4503599627370495_000000,
    },
    "case where floating point inaccuracy brings total below limit, negative"
  ],
];

for (const [arg, descr] of cases) {
  assert.sameValue(Temporal.Duration.compare(arg, arg), 0, descr);
}
