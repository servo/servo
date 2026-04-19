// Copyright (C) 2021 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-temporal.duration.prototype.total
description: Throw a TypeError if the receiver is invalid
features: [Symbol, Temporal]
---*/

const total = Temporal.Duration.prototype.total;

assert.sameValue(typeof total, "function");

const args = ["hour"];

assert.throws(TypeError, () => total.apply(undefined, args), "undefined");
assert.throws(TypeError, () => total.apply(null, args), "null");
assert.throws(TypeError, () => total.apply(true, args), "true");
assert.throws(TypeError, () => total.apply("", args), "empty string");
assert.throws(TypeError, () => total.apply(Symbol(), args), "symbol");
assert.throws(TypeError, () => total.apply(1, args), "1");
assert.throws(TypeError, () => total.apply({}, args), "plain object");
assert.throws(TypeError, () => total.apply(Temporal.Duration, args), "Temporal.Duration");
assert.throws(TypeError, () => total.apply(Temporal.Duration.prototype, args), "Temporal.Duration.prototype");
