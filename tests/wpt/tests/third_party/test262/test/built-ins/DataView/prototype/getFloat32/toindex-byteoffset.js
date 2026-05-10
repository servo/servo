// Copyright (C) 2016 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-dataview.prototype.getfloat32
description: >
  ToIndex conversions on byteOffset
info: |
  24.2.4.5 DataView.prototype.getFloat32 ( byteOffset [ , littleEndian ] )

  1. Let v be the this value.
  2. If littleEndian is not present, let littleEndian be false.
  3. Return ? GetViewValue(v, byteOffset, littleEndian, "Float32").

  24.2.1.1 GetViewValue ( view, requestIndex, isLittleEndian, type )

  ...
  4. Let getIndex be ? ToIndex(requestIndex).
  ...
features: [DataView.prototype.setUint8]
---*/

var buffer = new ArrayBuffer(12);
var sample = new DataView(buffer, 0);

sample.setUint8(0, 75);
sample.setUint8(1, 75);
sample.setUint8(2, 76);
sample.setUint8(3, 76);
sample.setUint8(4, 75);
sample.setUint8(5, 75);
sample.setUint8(6, 76);
sample.setUint8(7, 76);

var obj1 = {
  valueOf: function() {
    return 3;
  }
};

var obj2 = {
  toString: function() {
    return 2;
  }
};

assert.sameValue(sample.getFloat32(-0), 13323340, "-0");
assert.sameValue(sample.getFloat32(obj1), 53292336, "object's valueOf");
assert.sameValue(sample.getFloat32(obj2), 53554476, "object's toString");
assert.sameValue(sample.getFloat32(""), 13323340, "the Empty string");
assert.sameValue(sample.getFloat32("0"), 13323340, "string '0'");
assert.sameValue(sample.getFloat32("2"), 53554476, "string '2'");
assert.sameValue(sample.getFloat32(true), 13388875, "true");
assert.sameValue(sample.getFloat32(false), 13323340, "false");
assert.sameValue(sample.getFloat32(NaN), 13323340, "NaN");
assert.sameValue(sample.getFloat32(null), 13323340, "null");
assert.sameValue(sample.getFloat32(0.1), 13323340, "0.1");
assert.sameValue(sample.getFloat32(0.9), 13323340, "0.9");
assert.sameValue(sample.getFloat32(1.1), 13388875, "1.1");
assert.sameValue(sample.getFloat32(1.9), 13388875, "1.9");
assert.sameValue(sample.getFloat32(-0.1), 13323340, "-0.1");
assert.sameValue(sample.getFloat32(-0.99999), 13323340, "-0.99999");
assert.sameValue(sample.getFloat32(undefined), 13323340, "undefined");
assert.sameValue(sample.getFloat32(), 13323340, "no arg");
