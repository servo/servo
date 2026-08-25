// Copyright (C) 2022 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-get-temporal.plaindatetime.prototype.monthsinyear
description: Checking months in year for a "normal" case (non-undefined, non-boundary case, etc.)
features: [Temporal]
---*/

const datetime = new Temporal.PlainDateTime(1976, 11, 18, 15, 23, 30, 123, 456, 789, "iso8601");
assert.sameValue(datetime.monthsInYear, 12, "check months in year information");
assert.sameValue((new Temporal.PlainDateTime(1234, 8, 23, 5, 30, 13)).monthsInYear, 12);
