// Copyright (C) 2023 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-temporal.duration.prototype.subtract
description: >
    Duration-like argument performs the range check with minimal floating point
    precision loss
features: [Temporal]
---*/

// Based on a test case by André Bargull

const instance = new Temporal.Duration();

const balanceFailCases = [
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

// Adding a duration, even to a zero duration, causes rebalancing to the current
// largestUnit. These cases will not fail when converting the property bag to a
// duration, but they will fail during balancing after the addition when storing
// the resulting duration, because:
// 9007199254740991.975424 seconds balances into 9007199254740991975 ms, 424 µs
// ℝ(𝔽(9007199254740991975)) ms = 9007199254740992000 ms
// which is once again above the limit due to floating point inaccuracy.

for (const [arg, descr] of balanceFailCases) {
  assert.throws(RangeError, () => instance.subtract(arg), descr + ': ℝ(𝔽(x)) operation after balancing brings total over limit')
}

// These cases will balance to a largestUnit of seconds, which will not be
// inaccurate.

const balanceSuccessCases = [
  [
    {
      seconds: 2,
      milliseconds: 4503599627370496_500,  // ℝ(𝔽(4503599627370496500)) = 4503599627370496512
      microseconds: 4503599627370493_500000,  // ℝ(𝔽(4503599627370493500000)) = 4503599627370493378560
    },
    // 1 + 4503599627370496512 / 1000 + 4503599627370493378560 / 1000000 is
    // 9007199254740991.89056, which is below the limit of 2**53
    "-PT9007199254740991.89056S",
    "case where floating point inaccuracy brings total below limit, positive"
  ],
  [
    {
      seconds: -2,
      milliseconds: -4503599627370496_500,
      microseconds: -4503599627370493_500000,
    },
    "PT9007199254740991.89056S",
    "case where floating point inaccuracy brings total below limit, negative"
  ],
];

for (const [arg, string, descr] of balanceSuccessCases) {
  const result = instance.subtract(arg);
  assert.sameValue(result.toString(), string, descr);
}
