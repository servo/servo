// Copyright (C) 2017 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-dataview.prototype.getbigint64
description: >
  Detached buffer is checked before out of range byteOffset's value
info: |
  24.2.4.8 DataView.prototype.getBigInt64 ( byteOffset [ , littleEndian ] )

  1. Let v be the this value.
  2. If littleEndian is not present, let littleEndian be undefined.
  3. Return ? GetViewValue(v, byteOffset, littleEndian, "Int64").

  24.3.1.1 GetViewValue ( view, requestIndex, isLittleEndian, type )

  ...
  6. Let buffer be view.[[ViewedArrayBuffer]].
  7. If IsDetachedBuffer(buffer) is true, throw a TypeError exception.
  ...
  11. If getIndex + elementSize > viewSize, throw a RangeError exception.
  ...
includes: [detachArrayBuffer.js]
features: [DataView, ArrayBuffer, BigInt, arrow-function]
---*/

var sample;
var buffer = new ArrayBuffer(12);

sample = new DataView(buffer, 0);

$DETACHBUFFER(buffer);

assert.throws(TypeError, () => sample.getBigInt64(13),
  "detached DataView access should throw");
