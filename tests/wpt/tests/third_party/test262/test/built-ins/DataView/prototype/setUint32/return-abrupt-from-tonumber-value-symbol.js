// Copyright (C) 2016 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-dataview.prototype.setuint32
description: >
  Return abrupt from ToNumber(symbol value)
info: |
  24.2.4.20 DataView.prototype.setUint32 ( byteOffset, value [ , littleEndian ] )

  1. Let v be the this value.
  2. If littleEndian is not present, let littleEndian be false.
  3. Return ? SetViewValue(v, byteOffset, littleEndian, "Uint32", value).

  24.2.1.2 SetViewValue ( view, requestIndex, isLittleEndian, type, value )

  ...
  7. Let numberValue be ? ToNumber(value).
  ...
features: [Symbol]
---*/

var buffer = new ArrayBuffer(8);
var sample = new DataView(buffer, 0);

var s = Symbol("1");

assert.throws(TypeError, function() {
  sample.setUint32(0, s);
});
