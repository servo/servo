// Copyright (C) 2021 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-get-temporal.duration.prototype.nanoseconds
description: Throw a TypeError if the receiver is invalid
features: [Symbol, Temporal]
---*/

const nanoseconds = Object.getOwnPropertyDescriptor(Temporal.Duration.prototype, "nanoseconds").get;

assert.sameValue(typeof nanoseconds, "function");

assert.throws(TypeError, () => nanoseconds.call(undefined), "undefined");
assert.throws(TypeError, () => nanoseconds.call(null), "null");
assert.throws(TypeError, () => nanoseconds.call(true), "true");
assert.throws(TypeError, () => nanoseconds.call(""), "empty string");
assert.throws(TypeError, () => nanoseconds.call(Symbol()), "symbol");
assert.throws(TypeError, () => nanoseconds.call(1), "1");
assert.throws(TypeError, () => nanoseconds.call({}), "plain object");
assert.throws(TypeError, () => nanoseconds.call(Temporal.Duration), "Temporal.Duration");
assert.throws(TypeError, () => nanoseconds.call(Temporal.Duration.prototype), "Temporal.Duration.prototype");
