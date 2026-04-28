// Copyright (C) 2016 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-dataview.prototype.getint8
description: >
  Return value from Buffer using a clean ArrayBuffer
info: |
  24.2.4.7 DataView.prototype.getInt8 ( byteOffset )

  1. Let v be the this value.
  2. Return ? GetViewValue(v, byteOffset, true, "Int8").

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
---*/

var buffer = new ArrayBuffer(4);
var sample = new DataView(buffer, 0);

assert.sameValue(sample.getInt8(0), 0, "sample.getInt8(0)");
assert.sameValue(sample.getInt8(1), 0, "sample.getInt8(1)");
assert.sameValue(sample.getInt8(2), 0, "sample.getInt8(2)");
assert.sameValue(sample.getInt8(3), 0, "sample.getInt8(3)");
