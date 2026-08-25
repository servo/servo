// Copyright (C) 2021 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-temporal.plaintime.prototype.since
description: Throw a TypeError if the receiver is invalid
features: [Symbol, Temporal]
---*/

const since = Temporal.PlainTime.prototype.since;

assert.sameValue(typeof since, "function");

const args = [new Temporal.PlainTime(12)];

assert.throws(TypeError, () => since.apply(undefined, args), "undefined");
assert.throws(TypeError, () => since.apply(null, args), "null");
assert.throws(TypeError, () => since.apply(true, args), "true");
assert.throws(TypeError, () => since.apply("", args), "empty string");
assert.throws(TypeError, () => since.apply(Symbol(), args), "symbol");
assert.throws(TypeError, () => since.apply(1, args), "1");
assert.throws(TypeError, () => since.apply({}, args), "plain object");
assert.throws(TypeError, () => since.apply(Temporal.PlainTime, args), "Temporal.PlainTime");
assert.throws(TypeError, () => since.apply(Temporal.PlainTime.prototype, args), "Temporal.PlainTime.prototype");
