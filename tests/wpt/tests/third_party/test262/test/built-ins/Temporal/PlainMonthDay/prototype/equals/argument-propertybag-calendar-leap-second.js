// Copyright (C) 2022 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-temporal.plainmonthday.prototype.equals
description: Leap second is a valid ISO string for a calendar in a property bag
features: [Temporal]
---*/

const instance = new Temporal.PlainMonthDay(11, 18);

const calendar = "2016-12-31T23:59:60";

const arg = { monthCode: "M11", day: 18, calendar };
const result = instance.equals(arg);
assert.sameValue(
  result,
  true,
  "leap second is a valid ISO string for calendar"
);
