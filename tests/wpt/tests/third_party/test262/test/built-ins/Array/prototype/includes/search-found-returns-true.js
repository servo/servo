// Copyright (C) 2016 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-array.prototype.includes
description: returns true for found index
info: |
  22.1.3.11 Array.prototype.includes ( searchElement [ , fromIndex ] )

  ...
  5. If n ≥ 0, then
    a. Let k be n.
  6. Else n < 0,
    a. Let k be len + n.
    b. If k < 0, let k be 0.
  7. Repeat, while k < len
    a. Let elementK be the result of ? Get(O, ! ToString(k)).
    b. If SameValueZero(searchElement, elementK) is true, return true.
    c. Increase k by 1.
  ...
features: [Symbol, Array.prototype.includes]
---*/

var symbol = Symbol("1");
var obj = {};
var array = [];

var sample = [42, "test262", null, undefined, true, false, 0, -1, "", symbol, obj, array];

assert.sameValue(sample.includes(42), true, "42");
assert.sameValue(sample.includes("test262"), true, "'test262'");
assert.sameValue(sample.includes(null), true, "null");
assert.sameValue(sample.includes(undefined), true, "undefined");
assert.sameValue(sample.includes(true), true, "true");
assert.sameValue(sample.includes(false), true, "false");
assert.sameValue(sample.includes(0), true, "0");
assert.sameValue(sample.includes(-1), true, "-1");
assert.sameValue(sample.includes(""), true, "the empty string");
assert.sameValue(sample.includes(symbol), true, "symbol");
assert.sameValue(sample.includes(obj), true, "obj");
assert.sameValue(sample.includes(array), true, "array");
