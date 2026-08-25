// Copyright (C) 2022 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-temporal.plainyearmonth.prototype.from
description: Empty object may be used as options
includes: [temporalHelpers.js]
features: [Temporal]
---*/

TemporalHelpers.assertPlainYearMonth(
  Temporal.PlainYearMonth.from({ year: 2021, monthCode: "M01" }, {}), 2021, 1, "M01",
  "options may be an empty plain object"
);

TemporalHelpers.assertPlainYearMonth(
  Temporal.PlainYearMonth.from({ year: 2021, monthCode: "M01" }, () => {}), 2021, 1, "M01",
  "options may be an empty function object"
);
