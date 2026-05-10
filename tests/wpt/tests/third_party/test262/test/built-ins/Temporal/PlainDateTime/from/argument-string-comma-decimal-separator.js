// Copyright (C) 2022 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-temporal.plaindatetime.from
description: Comma may be used as a decimal separator
features: [Temporal]
includes: [temporalHelpers.js]
---*/

TemporalHelpers.assertPlainDateTime(
  Temporal.PlainDateTime.from("1976-11-18T15:23:30,12"),
  1976, 11, "M11", 18, 15, 23, 30, 120, 0, 0,
  "comma decimal separator"
);
