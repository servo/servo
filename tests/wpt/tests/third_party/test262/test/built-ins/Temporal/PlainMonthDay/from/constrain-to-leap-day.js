// Copyright (C) 2025 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-temporal.plainmonthday.from
description: Properly constrain February 30 to February 29, not 28
features: [Temporal]
---*/

const md = Temporal.PlainMonthDay.from({ monthCode: "M02", day: 30 }, { overflow: "constrain" });
assert.sameValue(md.day, 29, "M02-30 should constrain to 29, not 28");
