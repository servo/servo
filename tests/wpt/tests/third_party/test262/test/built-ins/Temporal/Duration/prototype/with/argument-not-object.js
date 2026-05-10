// Copyright (C) 2022 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-temporal.duration.prototype.with
description: Passing a primitive to with() throws
features: [Symbol, Temporal]
---*/

const instance = new Temporal.Duration(0, 0, 0, 1, 2, 3, 4, 987, 654, 321);
assert.throws(TypeError, () => instance.with(undefined), "undefined");
assert.throws(TypeError, () => instance.with(null), "null");
assert.throws(TypeError, () => instance.with(true), "boolean");
assert.throws(TypeError, () => instance.with(""), "empty string");
assert.throws(TypeError, () => instance.with("P1D"), "duration string");
assert.throws(TypeError, () => instance.with("string"), "string");
assert.throws(TypeError, () => instance.with(Symbol()), "Symbol");
assert.throws(TypeError, () => instance.with(7), "number");
assert.throws(TypeError, () => instance.with(7n), "bigint");
