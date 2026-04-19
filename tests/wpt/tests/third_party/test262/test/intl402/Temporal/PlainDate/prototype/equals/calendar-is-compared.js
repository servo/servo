// Copyright (C) 2021 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-temporal.plaindate.protoype.equals
description: test if the calendar is compared
features: [Temporal]
---*/

const date1 = new Temporal.PlainDate(1914, 2, 23, "iso8601");
const date2 = new Temporal.PlainDate(1914, 2, 22, "gregory");
assert.sameValue(date1.equals(date2), false, "different ISO dates");
