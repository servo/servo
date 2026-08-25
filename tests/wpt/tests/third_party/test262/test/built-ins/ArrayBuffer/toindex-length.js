// Copyright (C) 2016 The V8 Project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-arraybuffer-length
description: >
  The `length` parameter is converted to a value numeric index value.
info: |
  ArrayBuffer( length )

  1. If NewTarget is undefined, throw a TypeError exception.
  2. Let byteLength be ? ToIndex(length).
  3. Return ? AllocateArrayBuffer(NewTarget, byteLength).

  ToIndex( value )

  1. If value is undefined, then
    a. Let index be 0.
  2. Else,
    a. Let integerIndex be ? ToInteger(value).
    b. If integerIndex < 0, throw a RangeError exception.
    c. Let index be ! ToLength(integerIndex).
    d. If SameValueZero(integerIndex, index) is false, throw a RangeError exception.
  3. Return index.
---*/

var obj1 = {
  valueOf: function() {
    return 42;
  }
};

var obj2 = {
  toString: function() {
    return 42;
  }
};

var buffer;

buffer = new ArrayBuffer(obj1);
assert.sameValue(buffer.byteLength, 42, "object's valueOf");

buffer = new ArrayBuffer(obj2);
assert.sameValue(buffer.byteLength, 42, "object's toString");

buffer = new ArrayBuffer("");
assert.sameValue(buffer.byteLength, 0, "the Empty string");

buffer = new ArrayBuffer("0");
assert.sameValue(buffer.byteLength, 0, "string '0'");

buffer = new ArrayBuffer("1");
assert.sameValue(buffer.byteLength, 1, "string '1'");

buffer = new ArrayBuffer(true);
assert.sameValue(buffer.byteLength, 1, "true");

buffer = new ArrayBuffer(false);
assert.sameValue(buffer.byteLength, 0, "false");

buffer = new ArrayBuffer(NaN);
assert.sameValue(buffer.byteLength, 0, "NaN");

buffer = new ArrayBuffer(null);
assert.sameValue(buffer.byteLength, 0, "null");

buffer = new ArrayBuffer(undefined);
assert.sameValue(buffer.byteLength, 0, "undefined");

buffer = new ArrayBuffer(0.1);
assert.sameValue(buffer.byteLength, 0, "0.1");

buffer = new ArrayBuffer(0.9);
assert.sameValue(buffer.byteLength, 0, "0.9");

buffer = new ArrayBuffer(1.1);
assert.sameValue(buffer.byteLength, 1, "1.1");

buffer = new ArrayBuffer(1.9);
assert.sameValue(buffer.byteLength, 1, "1.9");

buffer = new ArrayBuffer(-0.1);
assert.sameValue(buffer.byteLength, 0, "-0.1");

buffer = new ArrayBuffer(-0.99999);
assert.sameValue(buffer.byteLength, 0, "-0.99999");
