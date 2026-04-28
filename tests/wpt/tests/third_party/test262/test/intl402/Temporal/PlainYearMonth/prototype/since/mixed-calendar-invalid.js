// Copyright (C) 2021 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-temporal.plainyearmonth.prototype.since
description: Mixed calendars throw as invalid
features: [Temporal]
---*/

const ym1 = new Temporal.PlainYearMonth(2000, 1);
const ym2 = new Temporal.PlainYearMonth(2000, 1, "gregory");

assert.throws(RangeError, () => ym1.since(ym2), 'since throws with different calendars');
