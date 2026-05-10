// Copyright (C) 2022 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-temporal.plainyearmonth.prototype.equals
description: equals() takes the reference day into account
features: [Temporal]
---*/

const ym1 = new Temporal.PlainYearMonth(2000, 1, "iso8601", 1);
const ym2 = new Temporal.PlainYearMonth(2000, 1, "iso8601", 2);
assert.sameValue(ym1.equals(ym2), false);
