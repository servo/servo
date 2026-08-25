// Copyright (C) 2022 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-get-temporal.plaindatetime.prototype.dayofyear
description: Checking day of year for a "normal" case (non-undefined, non-boundary case, etc.)
features: [Temporal]
---*/

const datetime = new Temporal.PlainDateTime(1976, 11, 18, 15, 23, 30, 123, 456, 789, "iso8601");
assert.sameValue(datetime.dayOfYear, 323, "check day of year information");

assert.sameValue((new Temporal.PlainDateTime(1997, 1, 23, 5, 30, 13)).dayOfYear, 23);
assert.sameValue((new Temporal.PlainDateTime(1997, 2, 23, 5, 30, 13)).dayOfYear, 54);
assert.sameValue((new Temporal.PlainDateTime(1996, 3, 23, 5, 30, 13)).dayOfYear, 83);
assert.sameValue((new Temporal.PlainDateTime(1997, 3, 23, 5, 30, 13)).dayOfYear, 82);
assert.sameValue((new Temporal.PlainDateTime(1997, 12, 31, 5, 30, 13)).dayOfYear, 365);
assert.sameValue((new Temporal.PlainDateTime(1996, 12, 31, 5, 30, 13)).dayOfYear, 366);
assert.sameValue(Temporal.PlainDateTime.from("1997-01-23T05:30:13").dayOfYear, 23);
assert.sameValue(Temporal.PlainDateTime.from("1997-02-23T05:30:13").dayOfYear, 54);
assert.sameValue(Temporal.PlainDateTime.from("1996-03-23T05:30:13").dayOfYear, 83);
assert.sameValue(Temporal.PlainDateTime.from("1997-03-23T05:30:13").dayOfYear, 82);
assert.sameValue(Temporal.PlainDateTime.from("1997-12-31T05:30:13").dayOfYear, 365);
assert.sameValue(Temporal.PlainDateTime.from("1996-12-31T05:30:13").dayOfYear, 366);
