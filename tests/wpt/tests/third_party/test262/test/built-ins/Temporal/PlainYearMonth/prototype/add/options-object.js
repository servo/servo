// Copyright (C) 2022 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-temporal.plainyearmonth.prototype.add
description: Empty or a function object may be used as options
includes: [temporalHelpers.js]
features: [Temporal]
---*/

const instance = new Temporal.PlainYearMonth(2019, 10);

const result1 = instance.add({ months: 1 }, {});
TemporalHelpers.assertPlainYearMonth(
  result1, 2019, 11, "M11",
  "options may be an empty plain object"
);

const result2 = instance.add({ months: 1 }, () => {});
TemporalHelpers.assertPlainYearMonth(
  result2, 2019, 11, "M11",
  "options may be a function object"
);
