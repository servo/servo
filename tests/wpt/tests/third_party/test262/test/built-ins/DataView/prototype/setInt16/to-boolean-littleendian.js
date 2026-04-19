// Copyright (C) 2016 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-dataview.prototype.setint16
description: >
  Boolean littleEndian argument coerced in ToBoolean
info: |
  24.2.4.16 DataView.prototype.setInt16 ( byteOffset, value [ , littleEndian ] )

  1. Let v be the this value.
  2. If littleEndian is not present, let littleEndian be false.
  3. Return ? SetViewValue(v, byteOffset, littleEndian, "Int16", value).

  24.2.1.2 SetViewValue ( view, requestIndex, isLittleEndian, type, value )

  ...
  15. Let bufferIndex be getIndex + viewOffset.
  16. Return SetValueInBuffer(buffer, bufferIndex, type, numberValue, isLittleEndian).
features: [DataView.prototype.getInt16, Symbol]
---*/

var buffer = new ArrayBuffer(8);
var sample = new DataView(buffer, 0);

// False
sample.setInt16(0, 1);
assert.sameValue(sample.getInt16(0), 1, "no arg");
sample.setInt16(0, 2, undefined);
assert.sameValue(sample.getInt16(0), 2, "undefined");
sample.setInt16(0, 3, null);
assert.sameValue(sample.getInt16(0), 3, "null");
sample.setInt16(0, 4, 0);
assert.sameValue(sample.getInt16(0), 4, "0");
sample.setInt16(0, 5, "");
assert.sameValue(sample.getInt16(0), 5, "the empty string");

// True
sample.setInt16(0, 1536, {});
assert.sameValue(sample.getInt16(0), 6, "{}");
sample.setInt16(0, 1792, Symbol("1"));
assert.sameValue(sample.getInt16(0), 7, "symbol");
sample.setInt16(0, 2048, 1);
assert.sameValue(sample.getInt16(0), 8, "1");
sample.setInt16(0, 2304, "string");
assert.sameValue(sample.getInt16(0), 9, "string");
