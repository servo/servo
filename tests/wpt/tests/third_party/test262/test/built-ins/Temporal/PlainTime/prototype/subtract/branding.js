// Copyright (C) 2021 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-temporal.plaintime.prototype.subtract
description: Throw a TypeError if the receiver is invalid
features: [Symbol, Temporal]
---*/

const subtract = Temporal.PlainTime.prototype.subtract;

assert.sameValue(typeof subtract, "function");

const args = [new Temporal.Duration(0, 0, 0, 0, 5)];

assert.throws(TypeError, () => subtract.apply(undefined, args), "undefined");
assert.throws(TypeError, () => subtract.apply(null, args), "null");
assert.throws(TypeError, () => subtract.apply(true, args), "true");
assert.throws(TypeError, () => subtract.apply("", args), "empty string");
assert.throws(TypeError, () => subtract.apply(Symbol(), args), "symbol");
assert.throws(TypeError, () => subtract.apply(1, args), "1");
assert.throws(TypeError, () => subtract.apply({}, args), "plain object");
assert.throws(TypeError, () => subtract.apply(Temporal.PlainTime, args), "Temporal.PlainTime");
assert.throws(TypeError, () => subtract.apply(Temporal.PlainTime.prototype, args), "Temporal.PlainTime.prototype");
