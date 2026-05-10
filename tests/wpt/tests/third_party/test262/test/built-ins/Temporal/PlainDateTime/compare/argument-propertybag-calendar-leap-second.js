// Copyright (C) 2022 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-temporal.plaindatetime.compare
description: Leap second is a valid ISO string for a calendar in a property bag
features: [Temporal]
---*/

const calendar = "2016-12-31T23:59:60";

const arg = { year: 1976, monthCode: "M11", day: 18, calendar };
const result1 = Temporal.PlainDateTime.compare(arg, new Temporal.PlainDateTime(1976, 11, 18));
assert.sameValue(result1, 0, "leap second is a valid ISO string for calendar (first argument)");
const result2 = Temporal.PlainDateTime.compare(new Temporal.PlainDateTime(1976, 11, 18), arg);
assert.sameValue(result2, 0, "leap second is a valid ISO string for calendar (second argument)");
