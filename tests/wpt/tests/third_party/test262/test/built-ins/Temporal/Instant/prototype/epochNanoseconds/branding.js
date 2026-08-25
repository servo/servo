// Copyright (C) 2020 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-get-temporal.instant.prototype.epochnanoseconds
description: Throw a TypeError if the receiver is invalid
features: [Symbol, Temporal]
---*/

const epochNanoseconds = Object.getOwnPropertyDescriptor(Temporal.Instant.prototype, "epochNanoseconds").get;

assert.sameValue(typeof epochNanoseconds, "function");

assert.throws(TypeError, () => epochNanoseconds.call(undefined), "undefined");
assert.throws(TypeError, () => epochNanoseconds.call(null), "null");
assert.throws(TypeError, () => epochNanoseconds.call(true), "true");
assert.throws(TypeError, () => epochNanoseconds.call(""), "empty string");
assert.throws(TypeError, () => epochNanoseconds.call(Symbol()), "symbol");
assert.throws(TypeError, () => epochNanoseconds.call(1), "1");
assert.throws(TypeError, () => epochNanoseconds.call({}), "plain object");
assert.throws(TypeError, () => epochNanoseconds.call(Temporal.Instant), "Temporal.Instant");
assert.throws(TypeError, () => epochNanoseconds.call(Temporal.Instant.prototype), "Temporal.Instant.prototype");
