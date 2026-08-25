// Copyright (C) 2023 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-temporal.duration.prototype.round
description: Rounding can cross unit boundaries up to the implicit largestUnit
includes: [temporalHelpers.js]
features: [Temporal]
---*/

const relativeTo = new Temporal.PlainDate(2022, 1, 1);
const roundingMode = "expand";

// Positive, date units
{
  const duration = new Temporal.Duration(1, 11, 0, 24);
  const result = duration.round({ smallestUnit: "months", roundingMode, relativeTo });
  TemporalHelpers.assertDuration(result, 2, 0, 0, 0, 0, 0, 0, 0, 0, 0, "1 year 12 months balances to 2 years");
}

// Negative, date units
{
  const duration = new Temporal.Duration(-1, -11, 0, -24);
  const result = duration.round({ smallestUnit: "months", roundingMode, relativeTo });
  TemporalHelpers.assertDuration(result, -2, 0, 0, 0, 0, 0, 0, 0, 0, 0, "-1 year -12 months balances to -2 years");
}

// Positive, time units
{
  const duration = new Temporal.Duration(0, 0, 0, 0, 1, 59, 59, 900);
  const result = duration.round({ smallestUnit: "seconds", roundingMode });
  TemporalHelpers.assertDuration(result, 0, 0, 0, 0, 2, 0, 0, 0, 0, 0, "1:59:60 balances to 2 hours");
}

// Negative, time units
{
  const duration = new Temporal.Duration(0, 0, 0, 0, -1, -59, -59, -900);
  const result = duration.round({ smallestUnit: "seconds", roundingMode });
  TemporalHelpers.assertDuration(result, 0, 0, 0, 0, -2, 0, 0, 0, 0, 0, "-1:59:60 balances to -2 hours");
}

// No balancing if smallest unit is largest unit
{
  const duration = new Temporal.Duration(0, 11, 0, 24);
  const result = duration.round({ smallestUnit: "months", roundingMode, relativeTo });
  TemporalHelpers.assertDuration(result, 0, 12, 0, 0, 0, 0, 0, 0, 0, 0, "12 months stays as is");
}
