// Copyright (C) 2021 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-temporal.duration.prototype.abs
description: Temporal.Duration.prototype.abs will return absolute value of the input duration.
info: |
  1. Let duration be the this value.
  2. Perform ? RequireInternalSlot(duration, [[InitializedTemporalDuration]]).
  3. Return ? CreateTemporalDuration(abs(duration.[[Years]]), abs(duration.[[Months]]), abs(duration.[[Weeks]]), abs(duration.[[Days]]), abs(duration.[[Hours]]), abs(duration.[[Minutes]]), abs(duration.[[Seconds]]), abs(duration.[[Milliseconds]]), abs(duration.[[Microseconds]]), abs(duration.[[Nanoseconds]])).

features: [Temporal]
includes: [temporalHelpers.js]
---*/

let d1 = new Temporal.Duration();
TemporalHelpers.assertDuration(
    d1.abs(), 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, "blank");

let d2 = new Temporal.Duration(1, 2, 3, 4, 5, 6, 7, 8, 9, 10);
TemporalHelpers.assertDuration(
    d2.abs(), 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, "positive");

let d3 = new Temporal.Duration(1e5, 2e5, 3e5, 4e5, 5e5, 6e5, 7e5, 8e5, 9e5, 10e5);
TemporalHelpers.assertDuration(
    d3.abs(), 1e5, 2e5, 3e5, 4e5, 5e5, 6e5, 7e5, 8e5, 9e5, 10e5, "large positive");

let d4 = new Temporal.Duration(-1, -2, -3, -4, -5, -6, -7, -8, -9, -10);
TemporalHelpers.assertDuration(
    d4.abs(), 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, "negative");

// Test with some zeros
let d5 = new Temporal.Duration(1, 0, 3, 0, 5, 0, 7, 0, 9, 0);
TemporalHelpers.assertDuration(
    d5.abs(), 1, 0, 3, 0, 5, 0, 7, 0, 9, 0, "some zeros");

let d6 = new Temporal.Duration(0, 2, 0, 4, 0, 6, 0, 8, 0, 10);
TemporalHelpers.assertDuration(
    d6.abs(), 0, 2, 0, 4, 0, 6, 0, 8, 0, 10, "other zeros");
