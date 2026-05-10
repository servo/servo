// Copyright (C) 2026 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-temporal.plainmonthday.from
description: The calendar name is case-insensitive
features: [Temporal]
---*/

const arg = "11-18[+01:00][u-ca=ISO8601]";
const result = Temporal.PlainMonthDay.from(arg);
assert.sameValue(result.calendarId, "iso8601", "Calendar is case-insensitive");
