// Copyright (C) 2016 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-dataview.prototype.getuint32
description: >
  Return values from Buffer using a custom offset
info: |
  24.2.4.12 DataView.prototype.getUint32 ( byteOffset [ , littleEndian ] )

  1. Let v be the this value.
  2. If littleEndian is not present, let littleEndian be false.
  3. Return ? GetViewValue(v, byteOffset, littleEndian, "Uint32").

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

sample.setUint8(0, 39);
sample.setUint8(1, 2);
sample.setUint8(2, 6);
sample.setUint8(3, 2);
sample.setUint8(4, 128);
sample.setUint8(5, 0);
sample.setUint8(6, 128);
sample.setUint8(7, 1);
sample.setUint8(8, 127);
sample.setUint8(9, 0);
sample.setUint8(10, 127);
sample.setUint8(11, 1);

sample = new DataView(buffer, 4);

assert.sameValue(sample.getUint32(0, false), 2147516417, "0, false");
assert.sameValue(sample.getUint32(1, false), 8388991, "1, false");
assert.sameValue(sample.getUint32(2, false), 2147581696, "2, false");
assert.sameValue(sample.getUint32(3, false), 25100415, "3, false");

assert.sameValue(sample.getUint32(0, true), 25165952, "0, true");
assert.sameValue(sample.getUint32(1, true), 2130804736, "1, true");
assert.sameValue(sample.getUint32(2, true), 8323456, "2, true");
assert.sameValue(sample.getUint32(3, true), 2130738945, "3, true");
