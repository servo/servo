// Copyright (C) 2021 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-temporal.plaindatetime.prototype.withplaintime
description: Throw a TypeError if the receiver is invalid
features: [Symbol, Temporal]
---*/

const withPlainTime = Temporal.PlainDateTime.prototype.withPlainTime;

assert.sameValue(typeof withPlainTime, "function");

assert.throws(TypeError, () => withPlainTime.call(undefined), "undefined");
assert.throws(TypeError, () => withPlainTime.call(null), "null");
assert.throws(TypeError, () => withPlainTime.call(true), "true");
assert.throws(TypeError, () => withPlainTime.call(""), "empty string");
assert.throws(TypeError, () => withPlainTime.call(Symbol()), "symbol");
assert.throws(TypeError, () => withPlainTime.call(1), "1");
assert.throws(TypeError, () => withPlainTime.call({}), "plain object");
assert.throws(TypeError, () => withPlainTime.call(Temporal.PlainDateTime), "Temporal.PlainDateTime");
assert.throws(TypeError, () => withPlainTime.call(Temporal.PlainDateTime.prototype), "Temporal.PlainDateTime.prototype");
