// Copyright (C) 2022 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-temporal.plaindatetime.prototype.equals
description: Checking a typical case (everything defined, no NaNs, nothing throws, etc.)
features: [Temporal]
---*/

const dt1 = new Temporal.PlainDateTime(1976, 11, 18, 15, 23, 30, 123, 456, 789);
const dt2 = new Temporal.PlainDateTime(2019, 10, 29, 10, 46, 38, 271, 986, 102);
const dt3 = new Temporal.PlainDateTime(2019, 10, 29, 10, 46, 38, 271, 986, 102);
const dt4 = new Temporal.PlainDateTime(2019, 10, 29, 15, 23, 30, 123, 456, 789);
const dt5 = new Temporal.PlainDateTime(1976, 11, 18, 10, 46, 38, 271, 986, 102);

assert.sameValue(dt1.equals(dt1), true, "equal");
assert.sameValue(dt1.equals(dt2), false, "unequal");
assert.sameValue(dt2.equals(dt3), true, "equal with different objects");
assert.sameValue(dt2.equals(dt4), false, "same date, different time");
assert.sameValue(dt2.equals(dt5), false, "same time, different date");
