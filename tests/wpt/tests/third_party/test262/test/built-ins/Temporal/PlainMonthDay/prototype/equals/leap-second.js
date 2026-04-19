// Copyright (C) 2022 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-temporal.plainmonthday.prototype.equals
description: Leap second is a valid ISO string for PlainMonthDay
features: [Temporal]
---*/

const instance = new Temporal.PlainMonthDay(12, 31);

let arg = "2016-12-31T23:59:60";
const result1 = instance.equals(arg);
assert.sameValue(
  result1,
  true,
  "leap second is a valid ISO string for PlainMonthDay"
);

arg = { year: 2016, month: 12, day: 31, hour: 23, minute: 59, second: 60 };
const result2 = instance.equals(arg);
assert.sameValue(
  result2,
  true,
  "second: 60 is ignored in property bag for PlainMonthDay"
);
