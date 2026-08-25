// Copyright (C) 2017 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-dataview.prototype.getbiguint64
description: Throws a TypeError if this is not Object
features: [DataView, ArrayBuffer, Symbol, BigInt, arrow-function]
---*/

var getBigUint64 = DataView.prototype.getBigUint64;

assert.throws(TypeError, () => getBigUint64.call(undefined),
  "undefined");

assert.throws(TypeError, () => getBigUint64.call(null), "null");

assert.throws(TypeError, () => getBigUint64.call(1), "1");

assert.throws(TypeError, () => getBigUint64.call("string"), "string");

assert.throws(TypeError, () => getBigUint64.call(true), "true");

assert.throws(TypeError, () => getBigUint64.call(false), "false");

var s = Symbol("1");
assert.throws(TypeError, () => getBigUint64.call(s), "symbol");
