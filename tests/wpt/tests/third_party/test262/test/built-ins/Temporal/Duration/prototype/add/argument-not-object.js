// Copyright (C) 2022 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-temporal.duration.prototype.add
description: Passing a primitive other than string to add() throws
features: [Symbol, Temporal]
---*/

const instance = new Temporal.Duration(0, 0, 0, 1, 2, 3, 4, 987, 654, 321);
assert.throws(TypeError, () => instance.add(undefined), "undefined");
assert.throws(TypeError, () => instance.add(null), "null");
assert.throws(TypeError, () => instance.add(true), "boolean");
assert.throws(RangeError, () => instance.add(""), "empty string");
assert.throws(TypeError, () => instance.add(Symbol()), "Symbol");
assert.throws(TypeError, () => instance.add(7), "number");
assert.throws(TypeError, () => instance.add(7n), "bigint");
assert.throws(TypeError, () => instance.add([]), "array");
assert.throws(TypeError, () => instance.add(() => {}), "function");
