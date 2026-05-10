// Copyright (C) 2022 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-temporal.plaintime.prototype.equals
description: Basic tests for equals()
features: [Temporal]
---*/

const t1 = Temporal.PlainTime.from("08:44:15.321");
const t1bis = Temporal.PlainTime.from("08:44:15.321");
const t2 = Temporal.PlainTime.from("14:23:30.123");
assert.sameValue(t1.equals(t1), true, "same object");
assert.sameValue(t1.equals(t1bis), true, "different object");
assert.sameValue(t1.equals(t2), false, "different times");
