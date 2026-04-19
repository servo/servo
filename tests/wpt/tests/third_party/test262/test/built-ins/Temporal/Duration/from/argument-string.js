// Copyright (C) 2022 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-temporal.duration.from
description: Basic string arguments.
includes: [temporalHelpers.js]
features: [Temporal]
---*/

TemporalHelpers.assertDuration(Temporal.Duration.from("P1D"),
  0, 0, 0, 1, 0, 0, 0, 0, 0, 0);
TemporalHelpers.assertDuration(Temporal.Duration.from("p1y1m1dt1h1m1s"),
  1, 1, 0, 1, 1, 1, 1, 0, 0, 0);
TemporalHelpers.assertDuration(Temporal.Duration.from("P1Y1M1W1DT1H1M1.1S"),
   1, 1, 1, 1, 1, 1, 1, 100, 0, 0);
TemporalHelpers.assertDuration(Temporal.Duration.from("P1Y1M1W1DT1H1M1.12S"),
   1, 1, 1, 1, 1, 1, 1, 120, 0, 0);
TemporalHelpers.assertDuration(Temporal.Duration.from("P1Y1M1W1DT1H1M1.123S"),
   1, 1, 1, 1, 1, 1, 1, 123, 0, 0);
TemporalHelpers.assertDuration(Temporal.Duration.from("P1Y1M1W1DT1H1M1.1234S"),
   1, 1, 1, 1, 1, 1, 1, 123, 400, 0);
TemporalHelpers.assertDuration(Temporal.Duration.from("P1Y1M1W1DT1H1M1.12345S"),
   1, 1, 1, 1, 1, 1, 1, 123, 450, 0);
TemporalHelpers.assertDuration(Temporal.Duration.from("P1Y1M1W1DT1H1M1.123456S"),
   1, 1, 1, 1, 1, 1, 1, 123, 456, 0);
TemporalHelpers.assertDuration(Temporal.Duration.from("P1Y1M1W1DT1H1M1.1234567S"),
   1, 1, 1, 1, 1, 1, 1, 123, 456, 700);
TemporalHelpers.assertDuration(Temporal.Duration.from("P1Y1M1W1DT1H1M1.12345678S"),
   1, 1, 1, 1, 1, 1, 1, 123, 456, 780);
TemporalHelpers.assertDuration(Temporal.Duration.from("P1Y1M1W1DT1H1M1.123456789S"),
   1, 1, 1, 1, 1, 1, 1, 123, 456, 789);
TemporalHelpers.assertDuration(Temporal.Duration.from("P1Y1M1W1DT1H1M1,12S"),
   1, 1, 1, 1, 1, 1, 1, 120, 0, 0);
TemporalHelpers.assertDuration(Temporal.Duration.from("P1DT0.5M"),
   0, 0, 0, 1, 0, 0, 30, 0, 0, 0);
TemporalHelpers.assertDuration(Temporal.Duration.from("P1DT0,5H"),
   0, 0, 0, 1, 0, 30, 0, 0, 0, 0);
TemporalHelpers.assertDuration(Temporal.Duration.from("+P1D"),
   0, 0, 0, 1, 0, 0, 0, 0, 0, 0);
TemporalHelpers.assertDuration(Temporal.Duration.from("-P1D"),
   0, 0, 0, -1, 0, 0, 0, 0, 0, 0);
TemporalHelpers.assertDuration(Temporal.Duration.from("-P1Y1M1W1DT1H1M1.123456789S"),
   -1, -1, -1, -1, -1, -1, -1, -123, -456, -789);
TemporalHelpers.assertDuration(Temporal.Duration.from("PT100M"),
   0, 0, 0, 0, 0, 100, 0, 0, 0, 0);
