// Copyright (C) 2022 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-temporal.plaindatetime.prototype.equals
description: Leap second is a valid ISO string for PlainDateTime
features: [Temporal]
---*/

const instance = new Temporal.PlainDateTime(2016, 12, 31, 23, 59, 59);

let arg = "2016-12-31T23:59:60";
const result1 = instance.equals(arg);
assert.sameValue(
  result1,
  true,
  "leap second is a valid ISO string for PlainDateTime"
);

arg = { year: 2016, month: 12, day: 31, hour: 23, minute: 59, second: 60 };
const result2 = instance.equals(arg);
assert.sameValue(
  result2,
  true,
  "second: 60 is ignored in property bag for PlainDateTime"
);
