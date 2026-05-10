// Copyright (C) 2022 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-temporal.plaindate.prototype.since
description: Tests calculations with roundingMode "trunc".
includes: [temporalHelpers.js]
features: [Temporal]
---*/

const earlier = Temporal.PlainDate.from("2019-01-08");
const later = Temporal.PlainDate.from("2021-09-07");

TemporalHelpers.assertDuration(
  later.since(earlier, { smallestUnit: "years", roundingIncrement: 4, roundingMode: "halfExpand" }),
  /* years = */ 4, 0, 0, 0, 0, 0, 0, 0, 0, 0, "years");

TemporalHelpers.assertDuration(
  later.since(earlier, { smallestUnit: "months", roundingIncrement: 10, roundingMode: "halfExpand" }),
  0, /* months = */ 30, 0, 0, 0, 0, 0, 0, 0, 0, "months");

TemporalHelpers.assertDuration(
  later.since(earlier, { smallestUnit: "weeks", roundingIncrement: 12, roundingMode: "halfExpand" }),
  0, 0, /* weeks = */ 144, 0, 0, 0, 0, 0, 0, 0, "weeks");

TemporalHelpers.assertDuration(
  later.since(earlier, { smallestUnit: "days", roundingIncrement: 100, roundingMode: "halfExpand" }),
  0, 0, 0, /* days = */ 1000, 0, 0, 0, 0, 0, 0, "days");
