// Copyright (C) 2022 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-temporal.plaindate.prototype.since
description: Tests calculations with higher smallestUnit than the default of "days"
includes: [compareArray.js, temporalHelpers.js]
features: [Temporal]
---*/

const earlier = Temporal.PlainDate.from("2019-01-08");
const later = Temporal.PlainDate.from("2021-09-07");

TemporalHelpers.assertDuration(
  later.since(earlier, { smallestUnit: "years", roundingMode: "halfExpand" }),
  /* years = */ 3, 0, 0, 0, 0, 0, 0, 0, 0, 0, "years");

TemporalHelpers.assertDuration(
  later.since(earlier, { smallestUnit: "months", roundingMode: "halfExpand" }),
  0, /* months = */ 32, 0, 0, 0, 0, 0, 0, 0, 0, "months");

TemporalHelpers.assertDuration(
  later.since(earlier, { smallestUnit: "weeks", roundingMode: "halfExpand" }),
  0, 0, /* weeks = */ 139, 0, 0, 0, 0, 0, 0, 0, "weeks");
