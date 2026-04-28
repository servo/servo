// Copyright (C) 2021 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-get-temporal.zoneddatetime.prototype.offset
description: Throw a TypeError if the receiver is invalid
features: [Symbol, Temporal]
---*/

const offset = Object.getOwnPropertyDescriptor(Temporal.ZonedDateTime.prototype, "offset").get;

assert.sameValue(typeof offset, "function");

assert.throws(TypeError, () => offset.call(undefined), "undefined");
assert.throws(TypeError, () => offset.call(null), "null");
assert.throws(TypeError, () => offset.call(true), "true");
assert.throws(TypeError, () => offset.call(""), "empty string");
assert.throws(TypeError, () => offset.call(Symbol()), "symbol");
assert.throws(TypeError, () => offset.call(1), "1");
assert.throws(TypeError, () => offset.call({}), "plain object");
assert.throws(TypeError, () => offset.call(Temporal.ZonedDateTime), "Temporal.ZonedDateTime");
assert.throws(TypeError, () => offset.call(Temporal.ZonedDateTime.prototype), "Temporal.ZonedDateTime.prototype");
