// Copyright (C) 2021 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-get-temporal.duration.prototype.microseconds
description: Throw a TypeError if the receiver is invalid
features: [Symbol, Temporal]
---*/

const microseconds = Object.getOwnPropertyDescriptor(Temporal.Duration.prototype, "microseconds").get;

assert.sameValue(typeof microseconds, "function");

assert.throws(TypeError, () => microseconds.call(undefined), "undefined");
assert.throws(TypeError, () => microseconds.call(null), "null");
assert.throws(TypeError, () => microseconds.call(true), "true");
assert.throws(TypeError, () => microseconds.call(""), "empty string");
assert.throws(TypeError, () => microseconds.call(Symbol()), "symbol");
assert.throws(TypeError, () => microseconds.call(1), "1");
assert.throws(TypeError, () => microseconds.call({}), "plain object");
assert.throws(TypeError, () => microseconds.call(Temporal.Duration), "Temporal.Duration");
assert.throws(TypeError, () => microseconds.call(Temporal.Duration.prototype), "Temporal.Duration.prototype");
