// Copyright (C) 2021 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-temporal.plainmonthday.prototype.tolocalestring
description: A time zone in resolvedOptions with a large offset still produces the correct string
locale: [en]
features: [Temporal]
---*/

const monthDay = new Temporal.PlainMonthDay(8, 4, "gregory");
const result = monthDay.toLocaleString("en", { timeZone: "Pacific/Apia" });
assert.sameValue(result, "8/4");
