// Copyright (C) 2021 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-temporal.plainmonthday.prototype.equals
description: Basic tests for equals() calendar handling
features: [Temporal]
---*/

const mdA = new Temporal.PlainMonthDay(2, 7, "iso8601");
const mdB = new Temporal.PlainMonthDay(2, 7, "gregory");
const mdC = new Temporal.PlainMonthDay(2, 7, "iso8601", 1974);
assert.sameValue(mdA.equals(mdC), false, "different year");
assert.sameValue(mdA.equals(mdB), false, "different calendar");
