// Copyright (C) 2022 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-temporal.plaindatetime.prototype.add
description: Strings with fractional duration units are rounded with the correct rounding mode
includes: [temporalHelpers.js]
features: [Temporal]
---*/

const datetime = new Temporal.PlainDateTime(2000, 5, 2);

TemporalHelpers.assertPlainDateTime(datetime.add("PT1.03125H"), 2000, 5, "M05", 2, 1, 1, 52, 500, 0, 0,
  "positive fractional units rounded with correct rounding mode");
TemporalHelpers.assertPlainDateTime(datetime.add("-PT1.03125H"), 2000, 5, "M05", 1, 22, 58, 7, 500, 0, 0,
  "negative fractional units rounded with correct rounding mode");
