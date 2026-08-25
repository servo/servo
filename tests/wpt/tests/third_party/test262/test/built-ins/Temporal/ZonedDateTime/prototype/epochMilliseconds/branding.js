// Copyright (C) 2021 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-get-temporal.zoneddatetime.prototype.epochmilliseconds
description: Throw a TypeError if the receiver is invalid
features: [Symbol, Temporal]
---*/

const epochMilliseconds = Object.getOwnPropertyDescriptor(Temporal.ZonedDateTime.prototype, "epochMilliseconds").get;

assert.sameValue(typeof epochMilliseconds, "function");

assert.throws(TypeError, () => epochMilliseconds.call(undefined), "undefined");
assert.throws(TypeError, () => epochMilliseconds.call(null), "null");
assert.throws(TypeError, () => epochMilliseconds.call(true), "true");
assert.throws(TypeError, () => epochMilliseconds.call(""), "empty string");
assert.throws(TypeError, () => epochMilliseconds.call(Symbol()), "symbol");
assert.throws(TypeError, () => epochMilliseconds.call(1), "1");
assert.throws(TypeError, () => epochMilliseconds.call({}), "plain object");
assert.throws(TypeError, () => epochMilliseconds.call(Temporal.ZonedDateTime), "Temporal.ZonedDateTime");
assert.throws(TypeError, () => epochMilliseconds.call(Temporal.ZonedDateTime.prototype), "Temporal.ZonedDateTime.prototype");
