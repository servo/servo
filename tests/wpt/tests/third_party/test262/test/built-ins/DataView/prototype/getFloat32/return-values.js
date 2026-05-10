// Copyright (C) 2016 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-dataview.prototype.getfloat32
description: >
  Return values from Buffer
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
features: [DataView.prototype.setUint8]
---*/

var buffer = new ArrayBuffer(8);
var sample = new DataView(buffer, 0);

sample.setUint8(0, 66);
sample.setUint8(1, 40);
sample.setUint8(2, 0);
sample.setUint8(3, 0);
sample.setUint8(4, 64);
sample.setUint8(5, 224);
sample.setUint8(6, 0);
sample.setUint8(7, 0);

assert.sameValue(sample.getFloat32(0, false), 42, "0, false");
assert.sameValue(sample.getFloat32(1, false), 7.105481567709626e-15, "1, false");
assert.sameValue(sample.getFloat32(2, false), 2.327276489550656e-41, "2, false");
assert.sameValue(sample.getFloat32(3, false), 5.95782781324968e-39, "3, false");
assert.sameValue(sample.getFloat32(4, false), 7, "4, false");

assert.sameValue(sample.getFloat32(0, true), 1.4441781973331565e-41, "0, true");
assert.sameValue(sample.getFloat32(1, true), 2.000009536743164, "1, true");
assert.sameValue(sample.getFloat32(2, true), -55340232221128655000, "2, true");
assert.sameValue(sample.getFloat32(3, true), 2.059411001342953e-38, "3, true");
assert.sameValue(sample.getFloat32(4, true), 8.04457422399591e-41, "4, true");

sample.setUint8(0, 75);
sample.setUint8(1, 75);
sample.setUint8(2, 76);
sample.setUint8(3, 76);
sample.setUint8(4, 75);
sample.setUint8(5, 75);
sample.setUint8(6, 76);
sample.setUint8(7, 76);

assert.sameValue(sample.getFloat32(0, false), 13323340, "0, false");
assert.sameValue(sample.getFloat32(1, false), 13388875, "1, false");
assert.sameValue(sample.getFloat32(2, false), 53554476, "2, false");
assert.sameValue(sample.getFloat32(3, false), 53292336, "3, false");
assert.sameValue(sample.getFloat32(4, false), 13323340, "4, false");
assert.sameValue(sample.getFloat32(0, true), 53554476, "0, true");
assert.sameValue(sample.getFloat32(1, true), 13388875, "1, true");
assert.sameValue(sample.getFloat32(2, true), 13323340, "2, true");
assert.sameValue(sample.getFloat32(3, true), 53292336, "3, true");
assert.sameValue(sample.getFloat32(4, true), 53554476, "4, true");
