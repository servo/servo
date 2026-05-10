// Copyright (C) 2022 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-temporal.duration.prototype.add
description: Strings with fractional duration units are rounded with the correct rounding mode
includes: [temporalHelpers.js]
features: [Temporal]
---*/

const blank = new Temporal.Duration();

TemporalHelpers.assertDuration(blank.add("PT1.03125H"), 0, 0, 0, 0, 1, 1, 52, 500, 0, 0,
  "positive fractional units rounded with correct rounding mode");
TemporalHelpers.assertDuration(blank.add("-PT1.03125H"), 0, 0, 0, 0, -1, -1, -52, -500, 0, 0,
  "negative fractional units rounded with correct rounding mode");
