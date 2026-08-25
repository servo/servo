// Copyright (C) 2021 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-temporal.zoneddatetime.prototype.round
description: Validity of increments depends on divisibility.
features: [Temporal]
---*/

const zdt = new Temporal.ZonedDateTime(217175010123456789n, "+01:00");

// valid hour increments divide into 24
const smallestUnit = "hour";
[
  1,
  2,
  3,
  4,
  6,
  8,
  12
].forEach(roundingIncrement => {
  assert(zdt.round({
    smallestUnit,
    roundingIncrement
  }) instanceof Temporal.ZonedDateTime);
});
[
  "minute",
  "second"
].forEach(smallestUnit => {
  // valid minutes/seconds increments divide into 60`, () => {
    [
      1,
      2,
      3,
      4,
      5,
      6,
      10,
      12,
      15,
      20,
      30
    ].forEach(roundingIncrement => {
      assert(zdt.round({
        smallestUnit,
        roundingIncrement
      }) instanceof Temporal.ZonedDateTime);
    });
  });
[
  "millisecond",
  "microsecond",
  "nanosecond"
].forEach(smallestUnit => {
  // valid increments divide into 1000`
    [
      1,
      2,
      4,
      5,
      8,
      10,
      20,
      25,
      40,
      50,
      100,
      125,
      200,
      250,
      500
    ].forEach(roundingIncrement => {
      assert(zdt.round({
        smallestUnit,
        roundingIncrement
      }) instanceof Temporal.ZonedDateTime);
    });
  });
