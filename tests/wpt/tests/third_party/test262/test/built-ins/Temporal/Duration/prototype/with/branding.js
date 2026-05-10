// Copyright (C) 2021 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-temporal.duration.prototype.with
description: Throw a TypeError if the receiver is invalid
features: [Symbol, Temporal]
---*/

const with_ = Temporal.Duration.prototype.with;

assert.sameValue(typeof with_, "function");

const args = [{ years: 3 }];

assert.throws(TypeError, () => with_.apply(undefined, args), "undefined");
assert.throws(TypeError, () => with_.apply(null, args), "null");
assert.throws(TypeError, () => with_.apply(true, args), "true");
assert.throws(TypeError, () => with_.apply("", args), "empty string");
assert.throws(TypeError, () => with_.apply(Symbol(), args), "symbol");
assert.throws(TypeError, () => with_.apply(1, args), "1");
assert.throws(TypeError, () => with_.apply({}, args), "plain object");
assert.throws(TypeError, () => with_.apply(Temporal.Duration, args), "Temporal.Duration");
assert.throws(TypeError, () => with_.apply(Temporal.Duration.prototype, args), "Temporal.Duration.prototype");
