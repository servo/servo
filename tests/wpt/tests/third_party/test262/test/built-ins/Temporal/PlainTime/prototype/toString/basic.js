// Copyright (C) 2022 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-temporal.plaintime.prototype.tostring
description: Basic tests for toString()
features: [Temporal]
---*/

assert.sameValue(new Temporal.PlainTime(15, 23).toString(), "15:23:00");
assert.sameValue(new Temporal.PlainTime(15, 23, 30).toString(), "15:23:30");
assert.sameValue(new Temporal.PlainTime(15, 23, 30, 123).toString(), "15:23:30.123");
assert.sameValue(new Temporal.PlainTime(15, 23, 30, 123, 400).toString(), "15:23:30.1234");
assert.sameValue(new Temporal.PlainTime(15, 23, 30, 123, 456).toString(), "15:23:30.123456");
assert.sameValue(new Temporal.PlainTime(15, 23, 30, 123, 456, 789).toString(), "15:23:30.123456789");
