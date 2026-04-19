// Copyright (C) 2021 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-temporal.plainyearmonth.from
description: The receiver is never called when calling from()
includes: [temporalHelpers.js]
features: [Temporal]
---*/

TemporalHelpers.checkSubclassingIgnoredStatic(
  Temporal.PlainYearMonth,
  "from",
  ["2000-05"],
  (result) => TemporalHelpers.assertPlainYearMonth(result, 2000, 5, "M05"),
);
