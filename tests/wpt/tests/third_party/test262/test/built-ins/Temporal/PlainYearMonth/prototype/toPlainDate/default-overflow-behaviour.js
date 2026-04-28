// Copyright (C) 2021 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-temporal.plainyearmonth.prototype.toplaindate
description: A nonexistent resulting date is constrained to an existing date
includes: [temporalHelpers.js]
features: [Temporal]
---*/

const febCommonYear = new Temporal.PlainYearMonth(2023, 2);
const result = febCommonYear.toPlainDate({ day: 29 });
// 2023-02-29 does not exist because 2023 is a common year
TemporalHelpers.assertPlainDate(result, 2023, 2, "M02", 28, "2023-02 + 29 = 2023-02-28");

const juneAnyYear = new Temporal.PlainYearMonth(1998, 6);
const result2 = juneAnyYear.toPlainDate({ day: 31 });
// 06-31 does not exist in any year
TemporalHelpers.assertPlainDate(result2, 1998, 6, "M06", 30, "1998-06 + 31 = 1998-06-31");
