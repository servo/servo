// Copyright (C) 2016 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-dataview.prototype.getuint16
description: >
  ToIndex conversions on byteOffset
info: |
  24.2.4.11 DataView.prototype.getUint16 ( byteOffset [ , littleEndian ] )

  1. Let v be the this value.
  2. If littleEndian is not present, let littleEndian be false.
  3. Return ? GetViewValue(v, byteOffset, littleEndian, "Uint16").

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
sample.setUint8(2, 1);
sample.setUint8(3, 127);
sample.setUint8(4, 255);
sample.setUint8(5, 1);

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

assert.sameValue(sample.getUint16(-0), 32767, "-0");
assert.sameValue(sample.getUint16(obj1), 383, "object's valueOf");
assert.sameValue(sample.getUint16(obj2), 32767, "object's toString");
assert.sameValue(sample.getUint16(""), 32767, "the Empty string");
assert.sameValue(sample.getUint16("0"), 32767, "string '0'");
assert.sameValue(sample.getUint16("2"), 383, "string '2'");
assert.sameValue(sample.getUint16(true), 65281, "true");
assert.sameValue(sample.getUint16(false), 32767, "false");
assert.sameValue(sample.getUint16(NaN), 32767, "NaN");
assert.sameValue(sample.getUint16(null), 32767, "null");
assert.sameValue(sample.getUint16(0.1), 32767, "0.1");
assert.sameValue(sample.getUint16(0.9), 32767, "0.9");
assert.sameValue(sample.getUint16(1.1), 65281, "1.1");
assert.sameValue(sample.getUint16(1.9), 65281, "1.9");
assert.sameValue(sample.getUint16(-0.1), 32767, "-0.1");
assert.sameValue(sample.getUint16(-0.99999), 32767, "-0.99999");
assert.sameValue(sample.getUint16(undefined), 32767, "undefined");
assert.sameValue(sample.getUint16(), 32767, "no arg");
