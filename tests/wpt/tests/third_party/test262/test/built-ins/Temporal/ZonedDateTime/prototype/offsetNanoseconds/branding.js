// Copyright (C) 2021 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-get-temporal.zoneddatetime.prototype.offsetnanoseconds
description: Throw a TypeError if the receiver is invalid
features: [Symbol, Temporal]
---*/

const offsetNanoseconds = Object.getOwnPropertyDescriptor(Temporal.ZonedDateTime.prototype, "offsetNanoseconds").get;

assert.sameValue(typeof offsetNanoseconds, "function");

assert.throws(TypeError, () => offsetNanoseconds.call(undefined), "undefined");
assert.throws(TypeError, () => offsetNanoseconds.call(null), "null");
assert.throws(TypeError, () => offsetNanoseconds.call(true), "true");
assert.throws(TypeError, () => offsetNanoseconds.call(""), "empty string");
assert.throws(TypeError, () => offsetNanoseconds.call(Symbol()), "symbol");
assert.throws(TypeError, () => offsetNanoseconds.call(1), "1");
assert.throws(TypeError, () => offsetNanoseconds.call({}), "plain object");
assert.throws(TypeError, () => offsetNanoseconds.call(Temporal.ZonedDateTime), "Temporal.ZonedDateTime");
assert.throws(TypeError, () => offsetNanoseconds.call(Temporal.ZonedDateTime.prototype), "Temporal.ZonedDateTime.prototype");
