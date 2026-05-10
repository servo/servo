// Copyright (C) 2022 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-temporal.plaintime.compare
description: Leap second is a valid ISO string for PlainTime
features: [Temporal]
---*/

let arg = "2016-12-31T23:59:60";
const result1 = Temporal.PlainTime.compare(arg, new Temporal.PlainTime(23, 59, 59));
assert.sameValue(result1, 0, "leap second is a valid ISO string for PlainTime (first argument)");
const result2 = Temporal.PlainTime.compare(new Temporal.PlainTime(23, 59, 59), arg);
assert.sameValue(result2, 0, "leap second is a valid ISO string for PlainTime (first argument)");

arg = { year: 2016, month: 12, day: 31, hour: 23, minute: 59, second: 60 };
const result3 = Temporal.PlainTime.compare(arg, new Temporal.PlainTime(23, 59, 59));
assert.sameValue(result3, 0, "second: 60 is ignored in property bag for PlainTime (first argument)");
const result4 = Temporal.PlainTime.compare(new Temporal.PlainTime(23, 59, 59), arg);
assert.sameValue(result4, 0, "second: 60 is ignored in property bag for PlainTime (second argument)");
