// Copyright (C) 2021 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-get-temporal.duration.prototype.seconds
description: Throw a TypeError if the receiver is invalid
features: [Symbol, Temporal]
---*/

const seconds = Object.getOwnPropertyDescriptor(Temporal.Duration.prototype, "seconds").get;

assert.sameValue(typeof seconds, "function");

assert.throws(TypeError, () => seconds.call(undefined), "undefined");
assert.throws(TypeError, () => seconds.call(null), "null");
assert.throws(TypeError, () => seconds.call(true), "true");
assert.throws(TypeError, () => seconds.call(""), "empty string");
assert.throws(TypeError, () => seconds.call(Symbol()), "symbol");
assert.throws(TypeError, () => seconds.call(1), "1");
assert.throws(TypeError, () => seconds.call({}), "plain object");
assert.throws(TypeError, () => seconds.call(Temporal.Duration), "Temporal.Duration");
assert.throws(TypeError, () => seconds.call(Temporal.Duration.prototype), "Temporal.Duration.prototype");
