// Copyright (C) 2025 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-temporal.plaindatetime.prototype.tolocalestring
description: dateStyle or timeStyle present but undefined
features: [Temporal]
---*/

const datetime = new Temporal.PlainDateTime(2000, 5, 2, 12, 34, 56, 987, 654, 321);
const defaultFormatter = new Intl.DateTimeFormat("en");
const expected = defaultFormatter.format(datetime);

const actualDate = datetime.toLocaleString("en", { dateStyle: undefined });
assert.sameValue(actualDate, expected, "dateStyle undefined is the same as being absent");

const actualTime = datetime.toLocaleString("en", { timeStyle: undefined });
assert.sameValue(actualTime, expected, "timeStyle undefined is the same as being absent");
