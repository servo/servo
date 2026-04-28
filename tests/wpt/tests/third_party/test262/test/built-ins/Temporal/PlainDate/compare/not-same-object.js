// Copyright (C) 2021 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-temporal.plaindate.compare
description: Dates are equal even if they are not the same object
features: [Temporal]
---*/

const date1 = new Temporal.PlainDate(1914, 2, 23);
const date2 = new Temporal.PlainDate(1914, 2, 23);

assert.sameValue(Temporal.PlainDate.compare(date1, date1), 0, "same object");
assert.sameValue(Temporal.PlainDate.compare(date1, date2), 0, "same date");
