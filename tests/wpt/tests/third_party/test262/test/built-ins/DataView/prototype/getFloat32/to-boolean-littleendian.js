// Copyright (C) 2016 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-dataview.prototype.getfloat32
description: >
  Boolean littleEndian argument coerced in ToBoolean
info: |
  24.2.4.5 DataView.prototype.getFloat32 ( byteOffset [ , littleEndian ] )

  1. Let v be the this value.
  2. If littleEndian is not present, let littleEndian be false.
  3. Return ? GetViewValue(v, byteOffset, littleEndian, "Float32").

  24.2.1.1 GetViewValue ( view, requestIndex, isLittleEndian, type )

  ...
  14. Let bufferIndex be getIndex + viewOffset.
  15. Return GetValueFromBuffer(buffer, bufferIndex, type, isLittleEndian).
  ...

  24.1.1.5 GetValueFromBuffer ( arrayBuffer, byteIndex, type [ , isLittleEndian
  ] )

  ...
  8. If isLittleEndian is false, reverse the order of the elements of rawValue.
  ...
features: [DataView.prototype.setUint8, Symbol]
---*/

var buffer = new ArrayBuffer(4);
var sample = new DataView(buffer, 0);

sample.setUint8(0, 75);
sample.setUint8(1, 75);
sample.setUint8(2, 76);
sample.setUint8(3, 76);

// False
assert.sameValue(sample.getFloat32(0), 13323340, "no arg");
assert.sameValue(sample.getFloat32(0, undefined), 13323340, "undefined");
assert.sameValue(sample.getFloat32(0, null), 13323340, "null");
assert.sameValue(sample.getFloat32(0, 0), 13323340, "0");
assert.sameValue(sample.getFloat32(0, ""), 13323340, "the empty string");

// True
assert.sameValue(sample.getFloat32(0, {}), 53554476, "{}");
assert.sameValue(sample.getFloat32(0, Symbol("1")), 53554476, "symbol");
assert.sameValue(sample.getFloat32(0, 1), 53554476, "1");
assert.sameValue(sample.getFloat32(0, "string"), 53554476, "string");
