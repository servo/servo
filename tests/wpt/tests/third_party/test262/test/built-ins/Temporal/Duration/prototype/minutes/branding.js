// Copyright (C) 2021 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-get-temporal.duration.prototype.minutes
description: Throw a TypeError if the receiver is invalid
features: [Symbol, Temporal]
---*/

const minutes = Object.getOwnPropertyDescriptor(Temporal.Duration.prototype, "minutes").get;

assert.sameValue(typeof minutes, "function");

assert.throws(TypeError, () => minutes.call(undefined), "undefined");
assert.throws(TypeError, () => minutes.call(null), "null");
assert.throws(TypeError, () => minutes.call(true), "true");
assert.throws(TypeError, () => minutes.call(""), "empty string");
assert.throws(TypeError, () => minutes.call(Symbol()), "symbol");
assert.throws(TypeError, () => minutes.call(1), "1");
assert.throws(TypeError, () => minutes.call({}), "plain object");
assert.throws(TypeError, () => minutes.call(Temporal.Duration), "Temporal.Duration");
assert.throws(TypeError, () => minutes.call(Temporal.Duration.prototype), "Temporal.Duration.prototype");
