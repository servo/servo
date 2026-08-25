// Copyright (C) 2016 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-array.prototype.includes
description: length value coerced on ToLength
info: |
  22.1.3.11 Array.prototype.includes ( searchElement [ , fromIndex ] )

  ...
  2. Let len be ? ToLength(? Get(O, "length")).
  ...

  7.1.15 ToLength ( argument )

  1. Let len be ? ToInteger(argument).
  2. If len ≤ +0, return +0.
  3. If len is +∞, return 253-1.
  4. Return min(len, 253-1).
features: [Array.prototype.includes]
---*/

var obj = {
  "0": "a",
  "1": "b"
};

obj.length = 0.1;
assert.sameValue([].includes.call(obj, "a"), false, "0.1");

obj.length = 0.99;
assert.sameValue([].includes.call(obj, "a"), false, "0.99");

obj.length = 1.00001;
assert.sameValue([].includes.call(obj, "a"), true, "1.00001");

obj.length = 1.1;
assert.sameValue([].includes.call(obj, "a"), true, "1.1");

obj.length = "0";
assert.sameValue([].includes.call(obj, "a"), false, "string '0'");

obj.length = "1";
assert.sameValue([].includes.call(obj, "a"), true, "string '1', item found");

obj.length = "1";
assert.sameValue([].includes.call(obj, "b"), false, "string '1', item not found");

obj.length = "2";
assert.sameValue([].includes.call(obj, "b"), true, "string '2', item found");

obj.length = "";
assert.sameValue([].includes.call(obj, "a"), false, "the empty string");

obj.length = undefined;
assert.sameValue([].includes.call(obj, "a"), false, "undefined");

obj.length = NaN;
assert.sameValue([].includes.call(obj, "a"), false, "NaN");

obj.length = [];
assert.sameValue([].includes.call(obj, "a"), false, "[]");

obj.length = [1];
assert.sameValue([].includes.call(obj, "a"), true, "[1]");

obj.length = null;
assert.sameValue([].includes.call(obj, "a"), false, "null");

obj.length = false;
assert.sameValue([].includes.call(obj, "a"), false, "false");

obj.length = true;
assert.sameValue([].includes.call(obj, "a"), true, "true");

obj.length = {
  valueOf: function() {
    return 2;
  }
};
assert.sameValue([].includes.call(obj, "b"), true, "ordinary object.valueOf");

obj.length = {
  toString: function() {
    return 2;
  }
};
assert.sameValue([].includes.call(obj, "b"), true, "ordinary object.toString");
