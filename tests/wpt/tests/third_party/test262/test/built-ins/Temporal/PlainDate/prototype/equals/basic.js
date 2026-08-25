// Copyright (C) 2021 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-temporal.plaindate.protoype.equals
description: basic tests
features: [Temporal]
---*/

const date1 = new Temporal.PlainDate(1976, 11, 18);
const date2 = new Temporal.PlainDate(1914, 2, 23);
const date3 = new Temporal.PlainDate(1914, 2, 23);
assert.sameValue(date1.equals(date2), false, "different dates");
assert.sameValue(date2.equals(date3), true, "same dates");
assert.sameValue(date2.equals(date2), true, "same objects");
