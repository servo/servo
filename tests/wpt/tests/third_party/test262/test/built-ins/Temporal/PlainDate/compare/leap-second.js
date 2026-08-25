// Copyright (C) 2022 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-temporal.plaindate.compare
description: Leap second is a valid ISO string for PlainDate
features: [Temporal]
---*/

let arg = "2016-12-31T23:59:60";
let result = Temporal.PlainDate.compare(arg, new Temporal.PlainDate(2016, 12, 31));
assert.sameValue(result, 0, "leap second is a valid ISO string for PlainDate (first argument)");
result = Temporal.PlainDate.compare(new Temporal.PlainDate(2016, 12, 31), arg);
assert.sameValue(result, 0, "leap second is a valid ISO string for PlainDate (second argument)");

arg = { year: 2016, month: 12, day: 31, hour: 23, minute: 59, second: 60 };
result = Temporal.PlainDate.compare(arg, new Temporal.PlainDate(2016, 12, 31));
assert.sameValue(result, 0, "second: 60 is ignored in property bag for PlainDate (first argument)");
result = Temporal.PlainDate.compare(new Temporal.PlainDate(2016, 12, 31), arg);
assert.sameValue(result, 0, "second: 60 is ignored in property bag for PlainDate (second argument)");
