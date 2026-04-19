// Copyright (C) 2021 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-get-temporal.duration.prototype.hours
description: Throw a TypeError if the receiver is invalid
features: [Symbol, Temporal]
---*/

const hours = Object.getOwnPropertyDescriptor(Temporal.Duration.prototype, "hours").get;

assert.sameValue(typeof hours, "function");

assert.throws(TypeError, () => hours.call(undefined), "undefined");
assert.throws(TypeError, () => hours.call(null), "null");
assert.throws(TypeError, () => hours.call(true), "true");
assert.throws(TypeError, () => hours.call(""), "empty string");
assert.throws(TypeError, () => hours.call(Symbol()), "symbol");
assert.throws(TypeError, () => hours.call(1), "1");
assert.throws(TypeError, () => hours.call({}), "plain object");
assert.throws(TypeError, () => hours.call(Temporal.Duration), "Temporal.Duration");
assert.throws(TypeError, () => hours.call(Temporal.Duration.prototype), "Temporal.Duration.prototype");
