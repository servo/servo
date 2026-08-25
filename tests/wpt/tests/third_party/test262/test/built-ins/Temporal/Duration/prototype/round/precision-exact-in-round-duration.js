// Copyright (C) 2022 André Bargull. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-temporal.duration.prototype.round
description: >
  RoundDuration computes on exact mathematical values.
includes: [temporalHelpers.js]
features: [Temporal]
---*/

{
  let duration = Temporal.Duration.from({
    hours: 100_000,
    nanoseconds: 5,
  });

  let rounded = duration.round({smallestUnit: "hours", roundingMode: "ceil"});

  // If RoundDuration() was implemented with float64, precision loss would lead
  // to computing an incorrect result.
  //
  // "PT100000H" with float64, but "PT100001H" with exact mathematical values.
  TemporalHelpers.assertDuration(
    rounded,
    0, 0, 0, 0,
    100001, 0, 0,
    0, 0, 0,
  );
}

{
  let duration = Temporal.Duration.from({
    days: 1000,
    nanoseconds: 5,
  });

  let rounded = duration.round({smallestUnit: "days", roundingMode: "ceil"});

  // If RoundDuration() was implemented with float64, precision loss would lead
  // to computing an incorrect result.
  //
  // "P1000D" with float64, but "P1001D" with exact mathematical values.
  TemporalHelpers.assertDuration(
    rounded,
    0, 0, 0, 1001,
    0, 0, 0,
    0, 0, 0,
  );
}
