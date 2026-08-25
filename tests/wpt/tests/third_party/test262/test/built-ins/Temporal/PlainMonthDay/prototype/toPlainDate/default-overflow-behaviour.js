// Copyright (C) 2021 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-temporal.plainmonthday.prototype.toplaindate
description: A nonexistent resulting date is constrained to an existing date
includes: [temporalHelpers.js]
features: [Temporal]
---*/

const leapDay = new Temporal.PlainMonthDay(2, 29);
const result = leapDay.toPlainDate({ year: 2023 });
// 2023-02-29 does not exist because 2023 is a common year
TemporalHelpers.assertPlainDate(result, 2023, 2, "M02", 28, "2023 + 02-29 = 2023-02-28");
