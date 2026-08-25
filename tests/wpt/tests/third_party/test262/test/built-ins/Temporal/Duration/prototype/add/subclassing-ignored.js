// Copyright (C) 2021 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-temporal.duration.prototype.add
description: Objects of a subclass are never created as return values.
includes: [temporalHelpers.js]
features: [Temporal]
---*/

TemporalHelpers.checkSubclassingIgnored(
  Temporal.Duration,
  [0, 0, 0, 4, 5, 6, 7, 987, 654, 321],
  "add",
  [{ nanoseconds: 1 }],
  (result) => TemporalHelpers.assertDuration(result, 0, 0, 0, 4, 5, 6, 7, 987, 654, 322),
);
