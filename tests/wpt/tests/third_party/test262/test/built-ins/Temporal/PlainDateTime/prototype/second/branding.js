// Copyright (C) 2021 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-get-temporal.plaindatetime.prototype.second
description: Throw a TypeError if the receiver is invalid
features: [Symbol, Temporal]
---*/

const second = Object.getOwnPropertyDescriptor(Temporal.PlainDateTime.prototype, "second").get;

assert.sameValue(typeof second, "function");

assert.throws(TypeError, () => second.call(undefined), "undefined");
assert.throws(TypeError, () => second.call(null), "null");
assert.throws(TypeError, () => second.call(true), "true");
assert.throws(TypeError, () => second.call(""), "empty string");
assert.throws(TypeError, () => second.call(Symbol()), "symbol");
assert.throws(TypeError, () => second.call(1), "1");
assert.throws(TypeError, () => second.call({}), "plain object");
assert.throws(TypeError, () => second.call(Temporal.PlainDateTime), "Temporal.PlainDateTime");
assert.throws(TypeError, () => second.call(Temporal.PlainDateTime.prototype), "Temporal.PlainDateTime.prototype");
