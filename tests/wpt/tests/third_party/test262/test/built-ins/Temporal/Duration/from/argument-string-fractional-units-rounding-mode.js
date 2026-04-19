// Copyright (C) 2022 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-temporal.duration.from
description: Strings with fractional duration units are rounded with the correct rounding mode
includes: [temporalHelpers.js]
features: [Temporal]
---*/

const resultPosHours = Temporal.Duration.from("PT1.03125H");
TemporalHelpers.assertDuration(resultPosHours, 0, 0, 0, 0, 1, 1, 52, 500, 0, 0,
  "positive fractional hours rounded with correct rounding mode");

const resultNegHours = Temporal.Duration.from("-PT1.03125H");
TemporalHelpers.assertDuration(resultNegHours, 0, 0, 0, 0, -1, -1, -52, -500, 0, 0,
  "negative fractional hours rounded with correct rounding mode");

const resultPosMinutes = Temporal.Duration.from('PT3.125M');
TemporalHelpers.assertDuration(resultPosMinutes, 0, 0, 0, 0, 0, 3, 7, 500, 0, 0,
  "positive fractional minutes rounded with correct rounding mode");

const resultNegMinutes = Temporal.Duration.from('-PT3,025M');
TemporalHelpers.assertDuration(resultNegMinutes, 0, 0, 0, 0, 0, -3, -1, -500, 0, 0,
  "negative fractional minutes rounded with correct rounding mode");


// The following input should not round, but may fail if an implementation does
// floating point arithmetic too early:

const resultPosSeconds = Temporal.Duration.from("PT46H66M71.50040904S");
TemporalHelpers.assertDuration(resultPosSeconds, 0, 0, 0, 0, 46, 66, 71, 500, 409, 40,
  "positive fractional seconds not rounded");

const resultNegSeconds = Temporal.Duration.from("-PT46H66M71.50040904S");
TemporalHelpers.assertDuration(resultNegSeconds, 0, 0, 0, 0, -46, -66, -71, -500, -409, -40,
  "negative fractional seconds not rounded");

const resultPosSecondsWithDate = Temporal.Duration.from('P11Y22M33W44D' + 'T55H66M77.987654321S');
TemporalHelpers.assertDuration(resultPosSecondsWithDate, 11, 22, 33, 44, 55, 66, 77, 987, 654, 321,
  "positive fractional seconds in datetime string not rounded");

const resultNegSecondsWithDate = Temporal.Duration.from('-P11Y22M33W44D' + 'T55H66M77.987654321S');
TemporalHelpers.assertDuration(resultNegSecondsWithDate, -11, -22, -33, -44, -55, -66, -77, -987, -654, -321,
  "negative fractional seconds in datetime string not rounded");
