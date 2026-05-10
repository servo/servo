// Copyright (C) 2021 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-get-temporal.duration.prototype.years
description: Throw a TypeError if the receiver is invalid
features: [Symbol, Temporal]
---*/

const years = Object.getOwnPropertyDescriptor(Temporal.Duration.prototype, "years").get;

assert.sameValue(typeof years, "function");

assert.throws(TypeError, () => years.call(undefined), "undefined");
assert.throws(TypeError, () => years.call(null), "null");
assert.throws(TypeError, () => years.call(true), "true");
assert.throws(TypeError, () => years.call(""), "empty string");
assert.throws(TypeError, () => years.call(Symbol()), "symbol");
assert.throws(TypeError, () => years.call(1), "1");
assert.throws(TypeError, () => years.call({}), "plain object");
assert.throws(TypeError, () => years.call(Temporal.Duration), "Temporal.Duration");
assert.throws(TypeError, () => years.call(Temporal.Duration.prototype), "Temporal.Duration.prototype");
