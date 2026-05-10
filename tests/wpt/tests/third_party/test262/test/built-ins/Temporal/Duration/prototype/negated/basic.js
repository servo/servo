// Copyright (C) 2021 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-temporal.duration.prototype.negated
description: Temporal.Duration.prototype.negated will return negated value of the input duration.
info: |
  3. Return ? CreateTemporalDuration(abs(duration.[[Years]]), abs(duration.[[Months]]), abs(duration.[[Weeks]]), abs(duration.[[Days]]), abs(duration.[[Hours]]), abs(duration.[[Minutes]]), abs(duration.[[Seconds]]), abs(duration.[[Milliseconds]]), abs(duration.[[Microseconds]]), abs(duration.[[Nanoseconds]])).
features: [Temporal]
includes: [temporalHelpers.js]
---*/

let d1 = new Temporal.Duration();
TemporalHelpers.assertDuration(
  d1.negated(), 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
  "blank");

let d2 = new Temporal.Duration(1, 2, 3, 4, 5, 6, 7, 8, 9, 10);
TemporalHelpers.assertDuration(
  d2.negated(), -1, -2, -3, -4, -5, -6, -7, -8, -9, -10,
  "positive values");

let d3 = new Temporal.Duration(1e5, 2e5, 3e5, 4e5, 5e5, 6e5, 7e5, 8e5, 9e5, 10e5);
TemporalHelpers.assertDuration(
  d3.negated(), -1e5, -2e5, -3e5, -4e5, -5e5, -6e5, -7e5, -8e5, -9e5, -10e5,
  "large positive values");

let d4 = new Temporal.Duration(-1, -2, -3, -4, -5, -6, -7, -8, -9, -10);
TemporalHelpers.assertDuration(
  d4.negated(), 1, 2, 3, 4, 5, 6, 7, 8, 9, 10,
  "negative values");

let d5 = new Temporal.Duration(-1e5, -2e5, -3e5, -4e5, -5e5, -6e5, -7e5, -8e5, -9e5, -10e5);
TemporalHelpers.assertDuration(
  d5.negated(), 1e5, 2e5, 3e5, 4e5, 5e5, 6e5, 7e5, 8e5, 9e5, 10e5,
  "large negative values");

let d6 = new Temporal.Duration(1, 0, 3, 0, 5, 0, 7, 0, 9, 0);
TemporalHelpers.assertDuration(
  d6.negated(), -1, 0, -3, 0, -5, 0, -7, 0, -9, 0,
  "some zeros with positive values");

let d7 = new Temporal.Duration(-1, 0, -3, 0, -5, 0, -7, 0, -9, 0);
TemporalHelpers.assertDuration(
  d7.negated(), 1, 0, 3, 0, 5, 0, 7, 0, 9, 0,
  "some zeros with negative values");

let d8 = new Temporal.Duration(0, -2, 0, -4, 0, -6, 0, -8, 0, -10);
TemporalHelpers.assertDuration(
  d8.negated(), 0, 2, 0, 4, 0, 6, 0, 8, 0, 10,
  "other zeros with negative values");
