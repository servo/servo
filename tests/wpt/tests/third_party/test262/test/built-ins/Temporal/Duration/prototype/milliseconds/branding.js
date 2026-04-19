// Copyright (C) 2021 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-get-temporal.duration.prototype.milliseconds
description: Throw a TypeError if the receiver is invalid
features: [Symbol, Temporal]
---*/

const milliseconds = Object.getOwnPropertyDescriptor(Temporal.Duration.prototype, "milliseconds").get;

assert.sameValue(typeof milliseconds, "function");

assert.throws(TypeError, () => milliseconds.call(undefined), "undefined");
assert.throws(TypeError, () => milliseconds.call(null), "null");
assert.throws(TypeError, () => milliseconds.call(true), "true");
assert.throws(TypeError, () => milliseconds.call(""), "empty string");
assert.throws(TypeError, () => milliseconds.call(Symbol()), "symbol");
assert.throws(TypeError, () => milliseconds.call(1), "1");
assert.throws(TypeError, () => milliseconds.call({}), "plain object");
assert.throws(TypeError, () => milliseconds.call(Temporal.Duration), "Temporal.Duration");
assert.throws(TypeError, () => milliseconds.call(Temporal.Duration.prototype), "Temporal.Duration.prototype");
