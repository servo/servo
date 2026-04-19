// Copyright (C) 2016 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-dataview.prototype.setint32
description: >
  Detached buffer is checked before out of range byteOffset's value
info: |
  24.2.4.17 DataView.prototype.setInt32 ( byteOffset, value [ , littleEndian ] )

  1. Let v be the this value.
  2. If littleEndian is not present, let littleEndian be false.
  3. Return ? SetViewValue(v, byteOffset, littleEndian, "Int32", value).

  24.2.1.2 SetViewValue ( view, requestIndex, isLittleEndian, type, value )

  ...
  9. Let buffer be the value of view's [[ViewedArrayBuffer]] internal slot.
  10. If IsDetachedBuffer(buffer) is true, throw a TypeError exception.
  ...
  14. If getIndex + elementSize > viewSize, throw a RangeError exception.
  ...
includes: [detachArrayBuffer.js]
---*/

var sample;
var buffer = new ArrayBuffer(12);

sample = new DataView(buffer, 0);

$DETACHBUFFER(buffer);

assert.throws(TypeError, function() {
  sample.setInt32(13, 0);
}, "13");
