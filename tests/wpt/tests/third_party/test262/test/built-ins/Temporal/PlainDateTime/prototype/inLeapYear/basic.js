// Copyright (C) 2022 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-get-temporal.plaindatetime.prototype.inleapyear
description: Basic test for inLeapYear
features: [Temporal]
---*/

assert.sameValue((new Temporal.PlainDateTime(1976, 11, 18, 15, 23, 30, 123, 456, 789)).inLeapYear,
  true, "leap year");
assert.sameValue((new Temporal.PlainDateTime(1977, 11, 18, 15, 23, 30, 123, 456, 789)).inLeapYear,
  false, "non-leap year");
assert.sameValue((new Temporal.PlainDateTime(1995, 8, 23, 5, 30, 13)).inLeapYear, false);
assert.sameValue((new Temporal.PlainDateTime(1996, 8, 23, 5, 30, 13)).inLeapYear, true);
assert.sameValue((new Temporal.PlainDateTime(1997, 8, 23, 5, 30, 13)).inLeapYear, false);
assert.sameValue((new Temporal.PlainDateTime(1998, 8, 23, 5, 30, 13)).inLeapYear, false);
assert.sameValue((new Temporal.PlainDateTime(1999, 8, 23, 5, 30, 13)).inLeapYear, false);
assert.sameValue((new Temporal.PlainDateTime(2000, 8, 23, 5, 30, 13)).inLeapYear, true);
assert.sameValue((new Temporal.PlainDateTime(2001, 8, 23, 5, 30, 13)).inLeapYear, false);
assert.sameValue((new Temporal.PlainDateTime(2002, 8, 23, 5, 30, 13)).inLeapYear, false);
assert.sameValue((new Temporal.PlainDateTime(2003, 8, 23, 5, 30, 13)).inLeapYear, false);
assert.sameValue((new Temporal.PlainDateTime(2004, 8, 23, 5, 30, 13)).inLeapYear, true);
assert.sameValue((new Temporal.PlainDateTime(2005, 8, 23, 5, 30, 13)).inLeapYear, false);
