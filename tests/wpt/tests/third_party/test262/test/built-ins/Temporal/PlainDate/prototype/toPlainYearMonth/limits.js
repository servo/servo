// Copyright (C) 2022 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-temporal.plaindate.prototype.toplainyearmonth
description: toPlainYearMonth works within the supported range
includes: [temporalHelpers.js]
features: [Temporal]
---*/

const min = Temporal.PlainDate.from('-271821-04-19');
TemporalHelpers.assertPlainYearMonth(min.toPlainYearMonth(),
  -271821, 4, "M04", "min");

const max = Temporal.PlainDate.from('+275760-09-13');
TemporalHelpers.assertPlainYearMonth(max.toPlainYearMonth(),
  275760, 9, "M09", "max");
