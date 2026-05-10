// Copyright (C) 2022 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-temporal.plaindatetime.prototype.since
description: Rounding argument cleanly divides the relevant smallest unit
features: [Temporal]
---*/

const earlier = new Temporal.PlainDateTime(2019, 1, 8, 8, 22, 36, 123, 456, 789);
const later = new Temporal.PlainDateTime(2021, 9, 7, 12, 39, 40, 987, 654, 321);

[1, 2, 3, 4, 6, 8, 12].forEach((roundingIncrement) => {
  const options = { smallestUnit: "hours", roundingIncrement };
  assert(
    later.since(earlier, options) instanceof Temporal.Duration,
    `valid hour increments divide into 24 (rounding increment = ${roundingIncrement}, smallest unit = hours)`
  );
});

["minutes", "seconds"].forEach((smallestUnit) => {
  [1, 2, 3, 4, 5, 6, 10, 12, 15, 20, 30].forEach((roundingIncrement) => {
    const options = { smallestUnit, roundingIncrement };
    assert(
      later.since(earlier, options) instanceof Temporal.Duration,
      `valid ${smallestUnit} increments divide into 60 (rounding increment = ${roundingIncrement})`
    );
  });
});

["milliseconds", "microseconds", "nanoseconds"].forEach((smallestUnit) => {
  [1, 2, 4, 5, 8, 10, 20, 25, 40, 50, 100, 125, 200, 250, 500].forEach((roundingIncrement) => {
    const options = { smallestUnit, roundingIncrement };
    assert(
      later.since(earlier, options) instanceof Temporal.Duration,
      `valid ${smallestUnit} increments divide into 1000 (rounding increment = ${roundingIncrement})`
    );
  });
});
