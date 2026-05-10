// Copyright (C) 2016 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-dataview.prototype.getuint32
description: >
  ToIndex conversions on byteOffset
info: |
  24.2.4.12 DataView.prototype.getUint32 ( byteOffset [ , littleEndian ] )

  1. Let v be the this value.
  2. If littleEndian is not present, let littleEndian be false.
  3. Return ? GetViewValue(v, byteOffset, littleEndian, "Uint32").

  24.2.1.1 GetViewValue ( view, requestIndex, isLittleEndian, type )

  ...
  4. Let getIndex be ? ToIndex(requestIndex).
  ...
features: [DataView.prototype.setUint8]
---*/

var buffer = new ArrayBuffer(8);
var sample = new DataView(buffer, 0);

sample.setUint8(0, 127);
sample.setUint8(1, 255);
sample.setUint8(2, 255);
sample.setUint8(3, 255);
sample.setUint8(4, 128);
sample.setUint8(5, 255);
sample.setUint8(6, 128);

var obj1 = {
  valueOf: function() {
    return 2;
  }
};

var obj2 = {
  toString: function() {
    return 3;
  }
};

assert.sameValue(sample.getUint32(-0), 2147483647, "-0");
assert.sameValue(sample.getUint32(obj1), 4294934783, "object's valueOf");
assert.sameValue(sample.getUint32(obj2), 4286644096, "object's toString");
assert.sameValue(sample.getUint32(""), 2147483647, "the Empty string");
assert.sameValue(sample.getUint32("0"), 2147483647, "string '0'");
assert.sameValue(sample.getUint32("2"), 4294934783, "string '2'");
assert.sameValue(sample.getUint32(true), 4294967168, "true");
assert.sameValue(sample.getUint32(false), 2147483647, "false");
assert.sameValue(sample.getUint32(NaN), 2147483647, "NaN");
assert.sameValue(sample.getUint32(null), 2147483647, "null");
assert.sameValue(sample.getUint32(0.1), 2147483647, "0.1");
assert.sameValue(sample.getUint32(0.9), 2147483647, "0.9");
assert.sameValue(sample.getUint32(1.1), 4294967168, "1.1");
assert.sameValue(sample.getUint32(1.9), 4294967168, "1.9");
assert.sameValue(sample.getUint32(-0.1), 2147483647, "-0.1");
assert.sameValue(sample.getUint32(-0.99999), 2147483647, "-0.99999");
assert.sameValue(sample.getUint32(undefined), 2147483647, "undefined");
assert.sameValue(sample.getUint32(), 2147483647, "no arg");
