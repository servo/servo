// Copyright (C) 2021 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-get-temporal.duration.prototype.sign
description: Throw a TypeError if the receiver is invalid
features: [Symbol, Temporal]
---*/

const sign = Object.getOwnPropertyDescriptor(Temporal.Duration.prototype, "sign").get;

assert.sameValue(typeof sign, "function");

assert.throws(TypeError, () => sign.call(undefined), "undefined");
assert.throws(TypeError, () => sign.call(null), "null");
assert.throws(TypeError, () => sign.call(true), "true");
assert.throws(TypeError, () => sign.call(""), "empty string");
assert.throws(TypeError, () => sign.call(Symbol()), "symbol");
assert.throws(TypeError, () => sign.call(1), "1");
assert.throws(TypeError, () => sign.call({}), "plain object");
assert.throws(TypeError, () => sign.call(Temporal.Duration), "Temporal.Duration");
assert.throws(TypeError, () => sign.call(Temporal.Duration.prototype), "Temporal.Duration.prototype");
