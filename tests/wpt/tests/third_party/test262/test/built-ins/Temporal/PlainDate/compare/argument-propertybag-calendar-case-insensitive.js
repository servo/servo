// Copyright (C) 2022 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-temporal.plaindate.compare
description: The calendar name is case-insensitive
features: [Temporal]
---*/

const arg = { year: 1976, monthCode: "M11", day: 18, calendar: "IsO8601" };
const result1 = Temporal.PlainDate.compare(arg, new Temporal.PlainDate(1976, 11, 18));
assert.sameValue(result1, 0, "Calendar is case-insensitive (first argument)");
const result2 = Temporal.PlainDate.compare(new Temporal.PlainDate(1976, 11, 18), arg);
assert.sameValue(result2, 0, "Calendar is case-insensitive (second argument)");

arg.calendar = "\u0130SO8601";
assert.throws(
  RangeError,
  () => Temporal.PlainDate.compare(arg, new Temporal.PlainDate(1976, 11, 18)),
  "calendar ID is capital dotted I is not lowercased (first argument)"
);
assert.throws(
  RangeError,
  () => Temporal.PlainDate.compare(new Temporal.PlainDate(1976, 11, 18), arg),
  "calendar ID is capital dotted I is not lowercased (second argument)"
);
