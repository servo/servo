// Copyright (C) 2020 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-temporal.instant.prototype.since
description: Throw a TypeError if the receiver is invalid
features: [Symbol, Temporal]
---*/

const since = Temporal.Instant.prototype.since;

assert.sameValue(typeof since, "function");

const args = [new Temporal.Instant(123456n)];

assert.throws(TypeError, () => since.apply(undefined, args), "undefined");
assert.throws(TypeError, () => since.apply(null, args), "null");
assert.throws(TypeError, () => since.apply(true, args), "true");
assert.throws(TypeError, () => since.apply("", args), "empty string");
assert.throws(TypeError, () => since.apply(Symbol(), args), "symbol");
assert.throws(TypeError, () => since.apply(1, args), "1");
assert.throws(TypeError, () => since.apply({}, args), "plain object");
assert.throws(TypeError, () => since.apply(Temporal.Instant, args), "Temporal.Instant");
assert.throws(TypeError, () => since.apply(Temporal.Instant.prototype, args), "Temporal.Instant.prototype");
