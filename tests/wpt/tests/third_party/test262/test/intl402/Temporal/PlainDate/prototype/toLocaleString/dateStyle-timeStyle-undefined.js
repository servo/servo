// Copyright (C) 2025 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-temporal.plaindate.prototype.tolocalestring
description: dateStyle or timeStyle present but undefined
features: [Temporal]
---*/

const date = new Temporal.PlainDate(2000, 5, 2);
const defaultFormatter = new Intl.DateTimeFormat("en");
const expected = defaultFormatter.format(date);

const actualDate = date.toLocaleString("en", { dateStyle: undefined });
assert.sameValue(actualDate, expected, "dateStyle undefined is the same as being absent");

const actualTime = date.toLocaleString("en", { timeStyle: undefined });
assert.sameValue(actualTime, expected, "timeStyle undefined is the same as being absent");
