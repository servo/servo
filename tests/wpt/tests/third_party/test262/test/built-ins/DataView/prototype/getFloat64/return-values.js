// Copyright (C) 2016 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-dataview.prototype.getfloat64
description: >
  Return values from Buffer
info: |
  24.2.4.6 DataView.prototype.getFloat64 ( byteOffset [ , littleEndian ] )

  1. Let v be the this value.
  2. If littleEndian is not present, let littleEndian be false.
  3. Return ? GetViewValue(v, byteOffset, littleEndian, "Float64").

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

var buffer = new ArrayBuffer(16);
var sample = new DataView(buffer, 0);

sample.setUint8(0, 67);
sample.setUint8(1, 67);
sample.setUint8(2, 68);
sample.setUint8(3, 68);
sample.setUint8(4, 67);
sample.setUint8(5, 67);
sample.setUint8(6, 68);
sample.setUint8(7, 68);
sample.setUint8(8, 67);
sample.setUint8(9, 67);
sample.setUint8(10, 68);
sample.setUint8(11, 68);
sample.setUint8(12, 0);
sample.setUint8(13, 0);
sample.setUint8(14, 0);
sample.setUint8(15, 0);

assert.sameValue(sample.getFloat64(0, false), 10846169068898440, "0, false");
assert.sameValue(sample.getFloat64(1, false), 11409110432516230, "1, false");
assert.sameValue(sample.getFloat64(2, false), 747563348316297500000, "2, false");
assert.sameValue(sample.getFloat64(3, false), 710670423110242000000, "3, false");
assert.sameValue(sample.getFloat64(4, false), 10846169068898440, "4, false");
assert.sameValue(sample.getFloat64(5, false), 11409110432516096, "5, false");
assert.sameValue(sample.getFloat64(6, false), 747563348314040600000, "6, false");
assert.sameValue(sample.getFloat64(7, false), 710670422532459300000, "7, false");
assert.sameValue(sample.getFloat64(8, false), 10846166811934720, "8, false");

assert.sameValue(sample.getFloat64(0, true), 747563348316297500000, "0, true");
assert.sameValue(sample.getFloat64(1, true), 11409110432516230, "1, true");
assert.sameValue(sample.getFloat64(2, true), 10846169068898440, "2, true");
assert.sameValue(sample.getFloat64(3, true), 710670423110242000000, "3, true");
assert.sameValue(sample.getFloat64(4, true), 747563348316297500000, "4, true");
assert.sameValue(sample.getFloat64(5, true), 2.254739805726094e-307, "5, true");
assert.sameValue(sample.getFloat64(6, true), 3.7084555987028e-310, "6, true");
assert.sameValue(sample.getFloat64(7, true), 1.44861546824e-312, "7, true");
assert.sameValue(sample.getFloat64(8, true), 5.65865417e-315, "8, true");
