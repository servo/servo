// Copyright (C) 2022 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-temporal.plainmonthday
description: referenceISOYear argument, if given, can cause RangeError
features: [Temporal]
---*/

assert.throws(RangeError, () => new Temporal.PlainMonthDay(9, 14, "iso8601", 275760), "after the maximum ISO date");
assert.throws(RangeError, () => new Temporal.PlainMonthDay(4, 18, "iso8601", -271821), "before the minimum ISO date")
