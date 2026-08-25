// Copyright (C) 2026 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-temporal.plaindatetime.from
description: Month code must be a string
features: [Temporal]
---*/

const monthCodeValues = [
  5, 5n, false, Symbol(), null, { toString: () => 5 }
];

const year = 2026;

for (const monthCode of monthCodeValues) {
  assert.throws(TypeError, () => Temporal.PlainDateTime.from({
    year,
    monthCode,
    day: 1, hour: 12, minute: 34
  }), typeof monthCode === 'symbol' ?
      "Symbol should be rejected as month code" :
      `month code ${monthCode} should be rejected`);
}
