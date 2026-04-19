// Copyright (C) 2021 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-temporal.plainmonthday.prototype.with
description: Objects of a subclass are never created as return values for with()
includes: [temporalHelpers.js]
features: [Temporal]
---*/

TemporalHelpers.checkSubclassingIgnored(
  Temporal.PlainMonthDay,
  [5, 2],
  "with",
  [{ day: 20 }],
  (result) => TemporalHelpers.assertPlainMonthDay(result, "M05", 20),
);
