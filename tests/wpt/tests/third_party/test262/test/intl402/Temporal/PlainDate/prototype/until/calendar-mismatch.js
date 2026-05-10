// Copyright (C) 2021 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-temporal.plaindate.prototype.until
description: RangeError thrown if calendars' IDs do not match
features: [Temporal]
---*/

const plainDate1 = new Temporal.PlainDate(2000, 1, 1, "gregory");
const plainDate2 = new Temporal.PlainDate(2000, 1, 1, "japanese");
assert.throws(RangeError, () => plainDate1.until(plainDate2));
