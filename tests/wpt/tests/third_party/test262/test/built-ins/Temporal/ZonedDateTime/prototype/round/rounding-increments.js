// Copyright (C) 2021 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-temporal.zoneddatetime.prototype.round
description: round() rounds to various increments.
includes: [temporalHelpers.js]
features: [Temporal]
---*/

const zdt = new Temporal.ZonedDateTime(217175010123456789n, "+01:00");
// "1976-11-18T16:00:00+01:00[+01:00]"
const expectedHours = new Temporal.ZonedDateTime(217177200000000000n, "+01:00");
// "1976-11-18T15:30:00+01:00[+01:00]"
const expectedMinutes = new Temporal.ZonedDateTime(217175400000000000n, "+01:00");
// "1976-11-18T15:23:30+01:00[+01:00]"
const expectedSeconds = new Temporal.ZonedDateTime(217175010000000000n, "+01:00");
// "1976-11-18T15:23:30.12+01:00[+01:00]");
const expectedMilliseconds = new Temporal.ZonedDateTime(217175010120000000n, "+01:00");
// "1976-11-18T15:23:30.12346+01:00[+01:00]");
const expectedMicroseconds = new Temporal.ZonedDateTime(217175010123460000n, "+01:00");
// "1976-11-18T15:23:30.12345679+01:00[+01:00]");
const expectedNanoseconds = new Temporal.ZonedDateTime(217175010123456790n, "+01:00");
// "1976-11-19T00:00:00+01:00[+01:00]");
const expected1Day = new Temporal.ZonedDateTime(217206000000000000n, "+01:00");

// rounds to an increment of hours
TemporalHelpers.assertZonedDateTimesEqual(zdt.round({
  smallestUnit: "hour",
  roundingIncrement: 4
}), expectedHours);

// rounds to an increment of minutes
TemporalHelpers.assertZonedDateTimesEqual(zdt.round({
  smallestUnit: "minute",
  roundingIncrement: 15
}), expectedMinutes);

// rounds to an increment of seconds
TemporalHelpers.assertZonedDateTimesEqual(zdt.round({
  smallestUnit: "second",
  roundingIncrement: 30
}), expectedSeconds);

// rounds to an increment of milliseconds
TemporalHelpers.assertZonedDateTimesEqual(zdt.round({
  smallestUnit: "millisecond",
  roundingIncrement: 10
}), expectedMilliseconds);

// rounds to an increment of microseconds
TemporalHelpers.assertZonedDateTimesEqual(zdt.round({
  smallestUnit: "microsecond",
  roundingIncrement: 10
}), expectedMicroseconds);

// rounds to an increment of nanoseconds
TemporalHelpers.assertZonedDateTimesEqual(zdt.round({
  smallestUnit: "nanosecond",
  roundingIncrement: 10
}), expectedNanoseconds);

// 1 day is a valid increment
TemporalHelpers.assertZonedDateTimesEqual(zdt.round({
  smallestUnit: "day",
  roundingIncrement: 1
}), expected1Day);

const unitsAndIncrements = {
   "hour": [1, 2, 4, 6, 8, 12],
   "minute": [1, 3, 5, 6, 10, 30],
   "second": [1, 3, 5, 6, 10, 30],
   "millisecond": [1, 5, 10, 20, 25, 50, 100, 500],
   "microsecond": [1, 5, 10, 20, 25, 50, 100, 500],
   "nanosecond": [1, 5, 10, 20, 25, 50, 100, 500],
};

// Just check that each combination of unit and increment doesn't throw
Object.entries(unitsAndIncrements).forEach(([unit, increments]) => {
  increments.forEach((increment) => {
    const result = zdt.round({ smallestUnit: unit, roundingMode: "ceil", roundingIncrement: increment });
    assert.sameValue(result instanceof Temporal.ZonedDateTime, true, `${unit} ${increment}`);
  })
});
