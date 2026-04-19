// Copyright (C) 2022 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-temporal.duration
description: Basic constructor tests.
includes: [temporalHelpers.js]
features: [Temporal]
---*/

TemporalHelpers.assertDuration(new Temporal.Duration(5, 5, 5, 5, 5, 5, 5, 5, 5, 0),
  5, 5, 5, 5, 5, 5, 5, 5, 5, 0, "positive");
TemporalHelpers.assertDuration(new Temporal.Duration(-5, -5, -5, -5, -5, -5, -5, -5, -5, 0),
  -5, -5, -5, -5, -5, -5, -5, -5, -5, 0, "negative");
TemporalHelpers.assertDuration(new Temporal.Duration(-0, -0, -0, -0, -0, -0, -0, -0, -0, -0),
  0, 0, 0, 0, 0, 0, 0, 0, 0, 0, "negative zero");
