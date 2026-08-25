// Copyright (C) 2021 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-temporal.plaindate.prototype.add
description: Objects of a subclass are never created as return values for add()
includes: [temporalHelpers.js]
features: [Temporal]
---*/

TemporalHelpers.checkSubclassingIgnored(
  Temporal.PlainDate,
  [2000, 5, 2],
  "add",
  [{ days: 1 }],
  (result) => TemporalHelpers.assertPlainDate(result, 2000, 5, "M05", 3),
);
