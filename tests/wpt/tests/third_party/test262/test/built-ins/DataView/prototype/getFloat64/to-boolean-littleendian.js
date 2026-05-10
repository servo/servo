// Copyright (C) 2016 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-dataview.prototype.getfloat64
description: >
  Boolean littleEndian argument coerced in ToBoolean
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
features: [DataView.prototype.setUint8, Symbol]
---*/

var buffer = new ArrayBuffer(8);
var sample = new DataView(buffer, 0);

sample.setUint8(0, 67);
sample.setUint8(1, 17);
sample.setUint8(2, 0);
sample.setUint8(3, 0);
sample.setUint8(4, 0);
sample.setUint8(5, 0);
sample.setUint8(6, 20);
sample.setUint8(7, 68);

// False
assert.sameValue(sample.getFloat64(0), 1196268651021585, "no arg");
assert.sameValue(sample.getFloat64(0, undefined), 1196268651021585, "undefined");
assert.sameValue(sample.getFloat64(0, null), 1196268651021585, "null");
assert.sameValue(sample.getFloat64(0, 0), 1196268651021585, "0");
assert.sameValue(sample.getFloat64(0, ""), 1196268651021585, "the empty string");

// True
assert.sameValue(sample.getFloat64(0, {}), 92233720368620160000, "{}");
assert.sameValue(sample.getFloat64(0, Symbol("1")), 92233720368620160000, "symbol");
assert.sameValue(sample.getFloat64(0, 1), 92233720368620160000, "1");
assert.sameValue(sample.getFloat64(0, "string"), 92233720368620160000, "string");
