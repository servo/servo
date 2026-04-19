// Copyright (C) 2021 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-temporal.instant.prototype.round
description: round() throws without required smallestUnit parameter.
includes: [temporalHelpers.js]
features: [Temporal]
---*/

const inst = Temporal.Instant.from("1976-11-18T14:23:30.123456789Z");

// rounds to an increment of hours
TemporalHelpers.assertInstantsEqual(inst.round({
  smallestUnit: "hour",
  roundingIncrement: 4
}), Temporal.Instant.from("1976-11-18T16:00:00Z"));

// rounds to an increment of minutes
TemporalHelpers.assertInstantsEqual(inst.round({
  smallestUnit: "minute",
  roundingIncrement: 15
}), Temporal.Instant.from("1976-11-18T14:30:00Z"));

// rounds to an increment of seconds
TemporalHelpers.assertInstantsEqual(inst.round({
  smallestUnit: "second",
  roundingIncrement: 30
}), Temporal.Instant.from("1976-11-18T14:23:30Z"));

// rounds to an increment of milliseconds
TemporalHelpers.assertInstantsEqual(inst.round({
  smallestUnit: "millisecond",
  roundingIncrement: 10
}), Temporal.Instant.from("1976-11-18T14:23:30.12Z"));

// rounds to an increment of microseconds
TemporalHelpers.assertInstantsEqual(inst.round({
  smallestUnit: "microsecond",
  roundingIncrement: 10
}), Temporal.Instant.from("1976-11-18T14:23:30.12346Z"));

// rounds to an increment of nanoseconds
TemporalHelpers.assertInstantsEqual(inst.round({
  smallestUnit: "nanosecond",
  roundingIncrement: 10
}), Temporal.Instant.from("1976-11-18T14:23:30.12345679Z"));

const unitsAndIncrements = {
   "hour": [1, 2, 4, 6, 8, 12, 24],
   "minute": [1, 5, 10, 20, 30, 40, 80, 120, 720, 1440],
   "second": [1, 5, 10, 20, 25, 30, 50, 100, 400, 86400],
   "millisecond": [1, 5, 10, 20, 25, 30, 50, 100, 86_400_000],
   "microsecond": [1, 5, 10, 20, 25, 30, 50, 100],
   "nanosecond": [1, 5, 10, 20, 25, 30, 50, 100]
};

// Just check that each combination of unit and increment doesn't throw
Object.entries(unitsAndIncrements).forEach(([unit, increments]) => {
  increments.forEach((increment) => {
    const result = inst.round({ smallestUnit: unit, roundingMode: "ceil", roundingIncrement: increment });
    assert.sameValue(result instanceof Temporal.Instant, true, `${unit} ${increment}`);
  })
});

