// Copyright (C) 2016 The V8 Project authors. All rights reserved.
// Copyright (C) 2016 Mozilla Corporation. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-sharedarraybuffer-length
description: >
  The `length` parameter is converted to a value numeric index value.
info: |
  SharedArrayBuffer( length )

  1. If NewTarget is undefined, throw a TypeError exception.
  2. Let byteLength be ? ToIndex(length).
  3. Return ? AllocateSharedArrayBuffer(NewTarget, byteLength).

  ToIndex( value )

  1. If value is undefined, then
    a. Let index be 0.
  2. Else,
    a. Let integerIndex be ? ToInteger(value).
    b. If integerIndex < 0, throw a RangeError exception.
    c. Let index be ! ToLength(integerIndex).
    d. If SameValueZero(integerIndex, index) is false, throw a RangeError exception.
  3. Return index.
features: [SharedArrayBuffer]
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

buffer = new SharedArrayBuffer(obj1);
assert.sameValue(buffer.byteLength, 42, "object's valueOf");

buffer = new SharedArrayBuffer(obj2);
assert.sameValue(buffer.byteLength, 42, "object's toString");

buffer = new SharedArrayBuffer("");
assert.sameValue(buffer.byteLength, 0, "the Empty string");

buffer = new SharedArrayBuffer("0");
assert.sameValue(buffer.byteLength, 0, "string '0'");

buffer = new SharedArrayBuffer("1");
assert.sameValue(buffer.byteLength, 1, "string '1'");

buffer = new SharedArrayBuffer(true);
assert.sameValue(buffer.byteLength, 1, "true");

buffer = new SharedArrayBuffer(false);
assert.sameValue(buffer.byteLength, 0, "false");

buffer = new SharedArrayBuffer(NaN);
assert.sameValue(buffer.byteLength, 0, "NaN");

buffer = new SharedArrayBuffer(null);
assert.sameValue(buffer.byteLength, 0, "null");

buffer = new SharedArrayBuffer(undefined);
assert.sameValue(buffer.byteLength, 0, "undefined");

buffer = new SharedArrayBuffer(0.1);
assert.sameValue(buffer.byteLength, 0, "0.1");

buffer = new SharedArrayBuffer(0.9);
assert.sameValue(buffer.byteLength, 0, "0.9");

buffer = new SharedArrayBuffer(1.1);
assert.sameValue(buffer.byteLength, 1, "1.1");

buffer = new SharedArrayBuffer(1.9);
assert.sameValue(buffer.byteLength, 1, "1.9");

buffer = new SharedArrayBuffer(-0.1);
assert.sameValue(buffer.byteLength, 0, "-0.1");

buffer = new SharedArrayBuffer(-0.99999);
assert.sameValue(buffer.byteLength, 0, "-0.99999");
