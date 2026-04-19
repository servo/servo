// Copyright (C) 2016 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-dataview.prototype.getint32
description: >
  Return values from Buffer
info: |
  24.2.4.9 DataView.prototype.getInt32 ( byteOffset [ , littleEndian ] )

  1. Let v be the this value.
  2. If littleEndian is not present, let littleEndian be false.
  3. Return ? GetViewValue(v, byteOffset, littleEndian, "Int32").

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

var buffer = new ArrayBuffer(12);
var sample = new DataView(buffer, 0);

sample.setUint8(0, 127);
sample.setUint8(1, 255);
sample.setUint8(2, 255);
sample.setUint8(3, 255);
sample.setUint8(4, 128);
sample.setUint8(5, 0);
sample.setUint8(6, 0);
sample.setUint8(7, 0);
sample.setUint8(8, 1);
sample.setUint8(9, 0);
sample.setUint8(10, 0);
sample.setUint8(11, 0);

assert.sameValue(sample.getInt32(0, false), 2147483647, "0, false"); // 2**32-1
assert.sameValue(sample.getInt32(1, false), -128, "1, false");
assert.sameValue(sample.getInt32(2, false), -32768, "2, false");
assert.sameValue(sample.getInt32(3, false), -8388608, "3, false");
assert.sameValue(sample.getInt32(4, false), -2147483648, "4, false");
assert.sameValue(sample.getInt32(5, false), 1, "5, false");
assert.sameValue(sample.getInt32(6, false), 256, "6, false");
assert.sameValue(sample.getInt32(7, false), 65536, "7, false");
assert.sameValue(sample.getInt32(8, false), 16777216, "8, false");

assert.sameValue(sample.getInt32(0, true), -129, "0, true");
assert.sameValue(sample.getInt32(1, true), -2130706433, "1, true");
assert.sameValue(sample.getInt32(2, true), 8454143, "2, true");
assert.sameValue(sample.getInt32(3, true), 33023, "3, true");
assert.sameValue(sample.getInt32(4, true), 128, "4, true");
assert.sameValue(sample.getInt32(5, true), 16777216, "5, true");
assert.sameValue(sample.getInt32(6, true), 65536, "6, true");
assert.sameValue(sample.getInt32(7, true), 256, "7, true");
assert.sameValue(sample.getInt32(8, true), 1, "8, true");
