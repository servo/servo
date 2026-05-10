// Copyright (C) 2022 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-temporal.plaindatetime.compare
description: Leap second is a valid ISO string for PlainDateTime
features: [Temporal]
---*/

let arg = "2016-12-31T23:59:60";
const result1 = Temporal.PlainDateTime.compare(arg, new Temporal.PlainDateTime(2016, 12, 31, 23, 59, 59));
assert.sameValue(result1, 0, "leap second is a valid ISO string for PlainDateTime (first argument)");
const result2 = Temporal.PlainDateTime.compare(new Temporal.PlainDateTime(2016, 12, 31, 23, 59, 59), arg);
assert.sameValue(result2, 0, "leap second is a valid ISO string for PlainDateTime (second argument)");

arg = { year: 2016, month: 12, day: 31, hour: 23, minute: 59, second: 60 };

const result3 = Temporal.PlainDateTime.compare(arg, new Temporal.PlainDateTime(2016, 12, 31, 23, 59, 59));
assert.sameValue(result3, 0, "second: 60 is constrained in property bag for PlainDateTime (first argument)");
const result4 = Temporal.PlainDateTime.compare(new Temporal.PlainDateTime(2016, 12, 31, 23, 59, 59), arg);
assert.sameValue(result4, 0, "second: 60 is constrained in property bag for PlainDateTime (second argument)");
