// Copyright (C) 2021 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-temporal.instant.prototype.round
description: Rounds to days by specifying increments of 86400 seconds in various units
includes: [temporalHelpers.js]
features: [Temporal]
---*/

const inst = Temporal.Instant.from("1976-11-18T14:23:30.123456789Z");
const expected = Temporal.Instant.from("1976-11-19T00:00:00Z");

TemporalHelpers.assertInstantsEqual(inst.round({
  smallestUnit: "hour",
  roundingIncrement: 24
}), expected);
TemporalHelpers.assertInstantsEqual(inst.round({
  smallestUnit: "minute",
  roundingIncrement: 1440
}), expected);
TemporalHelpers.assertInstantsEqual(inst.round({
  smallestUnit: "second",
  roundingIncrement: 86400
}), expected);
TemporalHelpers.assertInstantsEqual(inst.round({
  smallestUnit: "millisecond",
  roundingIncrement: 86400000
}), expected);
