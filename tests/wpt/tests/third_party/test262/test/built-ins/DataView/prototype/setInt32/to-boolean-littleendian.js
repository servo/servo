// Copyright (C) 2016 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-dataview.prototype.setint32
description: >
  Boolean littleEndian argument coerced in ToBoolean
info: |
  24.2.4.17 DataView.prototype.setInt32 ( byteOffset, value [ , littleEndian ] )

  1. Let v be the this value.
  2. If littleEndian is not present, let littleEndian be false.
  3. Return ? SetViewValue(v, byteOffset, littleEndian, "Int32", value).

  24.2.1.2 SetViewValue ( view, requestIndex, isLittleEndian, type, value )

  ...
  15. Let bufferIndex be getIndex + viewOffset.
  16. Return SetValueInBuffer(buffer, bufferIndex, type, numberValue, isLittleEndian).
features: [DataView.prototype.getInt32, Symbol]
---*/

var buffer = new ArrayBuffer(8);
var sample = new DataView(buffer, 0);

// False
sample.setInt32(0, 1);
assert.sameValue(sample.getInt32(0), 1, "no arg");
sample.setInt32(0, 2, undefined);
assert.sameValue(sample.getInt32(0), 2, "undefined");
sample.setInt32(0, 3, null);
assert.sameValue(sample.getInt32(0), 3, "null");
sample.setInt32(0, 4, 0);
assert.sameValue(sample.getInt32(0), 4, "0");
sample.setInt32(0, 5, "");
assert.sameValue(sample.getInt32(0), 5, "the empty string");

// True
sample.setInt32(0, 6, {});
assert.sameValue(sample.getInt32(0), 100663296, "{}");
sample.setInt32(0, 7, Symbol("1"));
assert.sameValue(sample.getInt32(0), 117440512, "symbol");
sample.setInt32(0, 8, 1);
assert.sameValue(sample.getInt32(0), 134217728, "1");
sample.setInt32(0, 9, "string");
assert.sameValue(sample.getInt32(0), 150994944, "string");
