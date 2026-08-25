// Copyright (C) 2022 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-temporal.plainyearmonth.prototype.with
description: Empty or a function object may be used as options
includes: [temporalHelpers.js]
features: [Temporal]
---*/

const instance = new Temporal.PlainYearMonth(2019, 10);

const result1 = instance.with({ year: 2020 }, {});
TemporalHelpers.assertPlainYearMonth(
  result1, 2020, 10, "M10",
  "options may be an empty plain object"
);

const result2 = instance.with({ year: 2020 }, () => {});
TemporalHelpers.assertPlainYearMonth(
  result2, 2020, 10, "M10",
  "options may be a function object"
);
