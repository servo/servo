// Copyright (C) 2022 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-temporal.plaindate.prototype.equals
description: The calendar name is case-insensitive
features: [Temporal]
---*/

const instance = new Temporal.PlainDate(1976, 11, 18);

const arg = { year: 1976, monthCode: "M11", day: 18, calendar: "IsO8601" };
const result = instance.equals(arg);
assert.sameValue(result, true, "Calendar is case-insensitive");

arg.calendar = "\u0130SO8601";
assert.throws(
  RangeError,
  () => instance.equals(arg),
  "calendar ID is capital dotted I is not lowercased"
);
