// Copyright (C) 2016 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-dataview.prototype.setfloat64
description: >
  Detached buffer is only checked after ToIndex(requestIndex)
info: |
  24.2.4.14 DataView.prototype.setFloat64 ( byteOffset, value [ , littleEndian ] )

  1. Let v be the this value.
  2. If littleEndian is not present, let littleEndian be false.
  3. Return ? SetViewValue(v, byteOffset, littleEndian, "Float64", value).

  24.2.1.2 SetViewValue ( view, requestIndex, isLittleEndian, type, value )

  ...
  4. Let getIndex be ? ToIndex(requestIndex).
  ...
  7. Let buffer be view.[[ViewedArrayBuffer]].
  8. If IsDetachedBuffer(buffer) is true, throw a TypeError exception.
  ...
includes: [detachArrayBuffer.js]
---*/

var buffer = new ArrayBuffer(12);
var sample = new DataView(buffer, 0);

$DETACHBUFFER(buffer);

assert.throws(RangeError, function() {
  sample.setFloat64(Infinity, 0);
}, "Infinity");

assert.throws(RangeError, function() {
  sample.setFloat64(-1, 0);
});
