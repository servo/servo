// Copyright (C) 2021 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-temporal.plaindate.protoype.tostring
description: basic tests
features: [Temporal]
---*/

const date1 = new Temporal.PlainDate(1976, 11, 18);
assert.sameValue(date1.toString(), "1976-11-18");

const date2 = new Temporal.PlainDate(1914, 2, 23);
assert.sameValue(date2.toString(), "1914-02-23");

const date3 = new Temporal.PlainDate(1996, 2, 29);
assert.sameValue(date3.toString(), "1996-02-29");
