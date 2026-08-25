// Copyright (C) 2021 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-get-temporal.duration.prototype.blank
description: Throw a TypeError if the receiver is invalid
features: [Symbol, Temporal]
---*/

const blank = Object.getOwnPropertyDescriptor(Temporal.Duration.prototype, "blank").get;

assert.sameValue(typeof blank, "function");

assert.throws(TypeError, () => blank.call(undefined), "undefined");
assert.throws(TypeError, () => blank.call(null), "null");
assert.throws(TypeError, () => blank.call(true), "true");
assert.throws(TypeError, () => blank.call(""), "empty string");
assert.throws(TypeError, () => blank.call(Symbol()), "symbol");
assert.throws(TypeError, () => blank.call(1), "1");
assert.throws(TypeError, () => blank.call({}), "plain object");
assert.throws(TypeError, () => blank.call(Temporal.Duration), "Temporal.Duration");
assert.throws(TypeError, () => blank.call(Temporal.Duration.prototype), "Temporal.Duration.prototype");
