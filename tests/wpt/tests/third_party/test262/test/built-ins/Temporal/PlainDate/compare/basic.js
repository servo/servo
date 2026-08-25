// Copyright (C) 2021 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-temporal.plaindate.compare
description: basic tests
features: [Temporal]
---*/

const d1 = Temporal.PlainDate.from("1976-11-18");
const d2 = Temporal.PlainDate.from("2019-06-30");
const d3 = Temporal.PlainDate.from("2019-06-30");
assert.sameValue(Temporal.PlainDate.compare(d1, d1), 0, "same object");
assert.sameValue(Temporal.PlainDate.compare(d1, d2), -1, "earlier");
assert.sameValue(Temporal.PlainDate.compare(d2, d1), 1, "later");
assert.sameValue(Temporal.PlainDate.compare(d2, d3), 0, "same date");
