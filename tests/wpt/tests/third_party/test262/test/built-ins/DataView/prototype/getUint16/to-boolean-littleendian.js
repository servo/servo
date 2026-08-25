// Copyright (C) 2016 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-dataview.prototype.getuint16
description: >
  Boolean littleEndian argument coerced in ToBoolean
info: |
  24.2.4.11 DataView.prototype.getUint16 ( byteOffset [ , littleEndian ] )

  1. Let v be the this value.
  2. If littleEndian is not present, let littleEndian be false.
  3. Return ? GetViewValue(v, byteOffset, littleEndian, "Uint16").

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

var buffer = new ArrayBuffer(2);
var sample = new DataView(buffer, 0);

sample.setUint8(0, 0);
sample.setUint8(1, 42);

// False
assert.sameValue(sample.getUint16(0), 42, "no arg");
assert.sameValue(sample.getUint16(0, undefined), 42, "undefined");
assert.sameValue(sample.getUint16(0, null), 42, "null");
assert.sameValue(sample.getUint16(0, 0), 42, "0");
assert.sameValue(sample.getUint16(0, ""), 42, "the empty string");

// True
assert.sameValue(sample.getUint16(0, {}), 10752, "{}");
assert.sameValue(sample.getUint16(0, Symbol("1")), 10752, "symbol");
assert.sameValue(sample.getUint16(0, 1), 10752, "1");
assert.sameValue(sample.getUint16(0, "string"), 10752, "string");
