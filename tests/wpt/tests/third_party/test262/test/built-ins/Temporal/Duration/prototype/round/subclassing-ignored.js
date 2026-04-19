// Copyright (C) 2021 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-temporal.duration.prototype.round
description: Objects of a subclass are never created as return values.
includes: [temporalHelpers.js]
features: [Temporal]
---*/

TemporalHelpers.checkSubclassingIgnored(
  Temporal.Duration,
  [0, 0, 0, 4, 5, 6, 7, 987, 654, 321],
  "round",
  [{ smallestUnit: 'seconds' }],
  (result) => TemporalHelpers.assertDuration(result, 0, 0, 0, 4, 5, 6, 8, 0, 0, 0),
);
