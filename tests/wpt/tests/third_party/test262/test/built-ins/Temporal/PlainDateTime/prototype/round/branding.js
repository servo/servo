// Copyright (C) 2021 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-temporal.plaindatetime.prototype.round
description: Throw a TypeError if the receiver is invalid
features: [Symbol, Temporal]
---*/

const round = Temporal.PlainDateTime.prototype.round;

assert.sameValue(typeof round, "function");

const args = ['hour'];

assert.throws(TypeError, () => round.apply(undefined, args), "undefined");
assert.throws(TypeError, () => round.apply(null, args), "null");
assert.throws(TypeError, () => round.apply(true, args), "true");
assert.throws(TypeError, () => round.apply("", args), "empty string");
assert.throws(TypeError, () => round.apply(Symbol(), args), "symbol");
assert.throws(TypeError, () => round.apply(1, args), "1");
assert.throws(TypeError, () => round.apply({}, args), "plain object");
assert.throws(TypeError, () => round.apply(Temporal.PlainDateTime, args), "Temporal.PlainDateTime");
assert.throws(TypeError, () => round.apply(Temporal.PlainDateTime.prototype, args), "Temporal.PlainDateTime.prototype");
