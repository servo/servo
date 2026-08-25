// Copyright (C) 2021 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-temporal.plaindatetime.prototype.subtract
description: Objects of a subclass are never created as return values for subtract()
includes: [temporalHelpers.js]
features: [Temporal]
---*/

TemporalHelpers.checkSubclassingIgnored(
  Temporal.PlainDateTime,
  [2000, 5, 2, 12, 34, 56, 987, 654, 321],
  "subtract",
  [{ nanoseconds: 1 }],
  (result) => TemporalHelpers.assertPlainDateTime(result, 2000, 5, "M05", 2, 12, 34, 56, 987, 654, 320),
);
