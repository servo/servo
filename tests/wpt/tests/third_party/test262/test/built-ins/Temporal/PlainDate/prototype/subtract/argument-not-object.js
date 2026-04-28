// Copyright (C) 2020 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-temporal.plaindate.prototype.subtract
description: Passing a primitive other than string to subtract() throws
features: [Symbol, Temporal]
---*/

const instance = new Temporal.PlainDate(2000, 5, 2);
assert.throws(TypeError, () => instance.subtract(undefined), "undefined");
assert.throws(TypeError, () => instance.subtract(null), "null");
assert.throws(TypeError, () => instance.subtract(true), "boolean");
assert.throws(RangeError, () => instance.subtract(""), "empty string");
assert.throws(TypeError, () => instance.subtract(Symbol()), "Symbol");
assert.throws(TypeError, () => instance.subtract(7), "number");
assert.throws(TypeError, () => instance.subtract(7n), "bigint");
assert.throws(TypeError, () => instance.subtract([]), "array");
assert.throws(TypeError, () => instance.subtract(() => {}), "function");
