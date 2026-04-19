// Copyright (C) 2025 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-temporal.plaintime.prototype.tolocalestring
description: timeStyle present but undefined
features: [Temporal]
---*/

const time = new Temporal.PlainTime(12, 34, 56, 987, 654, 321);
const defaultFormatter = new Intl.DateTimeFormat("en");
const expected = defaultFormatter.format(time);

const actual = time.toLocaleString("en", { timeStyle: undefined });
assert.sameValue(actual, expected, "timeStyle undefined is the same as being absent");
