// Copyright (C) 2020 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-temporal.plaindatetime.prototype.equals
description: Throw a TypeError if the receiver is invalid
features: [Symbol, Temporal]
---*/

const equals = Temporal.PlainDateTime.prototype.equals;

assert.sameValue(typeof equals, "function");

const args = [new Temporal.PlainDateTime(2022, 6, 22)];

assert.throws(TypeError, () => equals.apply(undefined, args), "undefined");
assert.throws(TypeError, () => equals.apply(null, args), "null");
assert.throws(TypeError, () => equals.apply(true, args), "true");
assert.throws(TypeError, () => equals.apply("", args), "empty string");
assert.throws(TypeError, () => equals.apply(Symbol(), args), "symbol");
assert.throws(TypeError, () => equals.apply(1, args), "1");
assert.throws(TypeError, () => equals.apply({}, args), "plain object");
assert.throws(TypeError, () => equals.apply(Temporal.PlainDateTime, args), "Temporal.PlainDateTime");
assert.throws(TypeError, () => equals.apply(Temporal.PlainDateTime.prototype, args), "Temporal.PlainDateTime.prototype");
