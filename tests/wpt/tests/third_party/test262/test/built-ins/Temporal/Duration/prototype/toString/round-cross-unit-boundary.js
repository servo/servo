// Copyright (C) 2023 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-temporal.duration.prototype.tostring
description: Rounding can cross unit boundaries up to days
features: [Temporal]
---*/

const roundingMode = "expand";

// Positive, time units
{
  const duration = new Temporal.Duration(0, 0, 0, 0, 1, 59, 59, 900);
  assert.sameValue(duration.toString({ fractionalSecondDigits: 0, roundingMode }), "PT2H0S", "1:59:60 balances to 2 hours");
}

// Negative, time units
{
  const duration = new Temporal.Duration(0, 0, 0, 0, -1, -59, -59, -900);
  assert.sameValue(duration.toString({ fractionalSecondDigits: 0, roundingMode }), "-PT2H0S", "-1:59:60 balances to -2 hours");
}

// Positive, date and time units
{
  const duration = new Temporal.Duration(1, 11, 0, 30, 23, 59, 59, 999, 999, 999);
  assert.sameValue(duration.toString({ fractionalSecondDigits: 8, roundingMode }), "P1Y11M31DT0.00000000S", "units balance only up to days (positive)");
}

// Negative, date and time units
{
  const duration = new Temporal.Duration(-1, -11, 0, -30, -23, -59, -59, -999, -999, -999);
  assert.sameValue(duration.toString({ fractionalSecondDigits: 8, roundingMode }), "-P1Y11M31DT0.00000000S", "units balance only up to days (negative)");
}

// No balancing if smallest unit is largest unit
{
  const duration = new Temporal.Duration(0, 0, 0, 0, 0, 0, 59, 900);
  assert.sameValue(duration.toString({ fractionalSecondDigits: 0, roundingMode }), "PT60S", "60 seconds stays as is");
}
