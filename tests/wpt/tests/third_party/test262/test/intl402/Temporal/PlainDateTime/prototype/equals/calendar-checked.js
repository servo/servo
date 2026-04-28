// Copyright (C) 2022 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-temporal.plaindatetime.prototype.equals
description: Calendar is taken into account if the ISO data is equal
features: [Temporal]
---*/

const dt1 = new Temporal.PlainDateTime(1976, 11, 18, 15, 23, 30, 123, 456, 789, "iso8601");
const dt2 = new Temporal.PlainDateTime(1976, 11, 18, 15, 23, 30, 123, 456, 789, "iso8601");
const dt3 = new Temporal.PlainDateTime(1976, 11, 18, 15, 23, 30, 123, 456, 789, "gregory");

assert.sameValue(dt1.equals(dt2), true, "same calendar string");
assert.sameValue(dt1.equals(dt3), false, "different calendar string");

const dt4 = new Temporal.PlainDateTime(1976, 11, 18, 15, 23, 30, 123, 456, 789, "iso8601");
const dt5 = new Temporal.PlainDateTime(2019, 10, 29, 10, 46, 38, 271, 986, 102, "iso8601");
const dt6 = new Temporal.PlainDateTime(2019, 10, 29, 10, 46, 38, 271, 986, 102, "gregory");
assert.sameValue(dt4.equals(dt5), false, "not equal same calendar");
assert.sameValue(dt4.equals(dt6), false, "not equal different calendar");
