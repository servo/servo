// Copyright (C) 2022 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-temporal.plaintime.compare
description: Basic tests for compare()
features: [Temporal]
---*/

const t1 = Temporal.PlainTime.from("08:44:15.321");
const t1bis = Temporal.PlainTime.from("08:44:15.321");
const t2 = Temporal.PlainTime.from("14:23:30.123");

assert.sameValue(Temporal.PlainTime.compare(t1, t1), 0, "same object");
assert.sameValue(Temporal.PlainTime.compare(t1, t1bis), 0, "different object");
assert.sameValue(Temporal.PlainTime.compare(t1, t2), -1, "before");
assert.sameValue(Temporal.PlainTime.compare(t2, t1), 1, "after");
