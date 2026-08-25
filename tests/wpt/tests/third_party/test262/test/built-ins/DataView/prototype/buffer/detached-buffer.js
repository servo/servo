// Copyright (C) 2016 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-get-dataview.prototype.buffer
description: The getter method does not throw with a detached buffer
info: |
  24.2.4.1 get DataView.prototype.buffer

  ...
  5. Let buffer be the value of O's [[ViewedArrayBuffer]] internal slot.
  6. Return buffer.
includes: [detachArrayBuffer.js]
---*/

var buffer = new ArrayBuffer(8);
var sample = new DataView(buffer, 0);

$DETACHBUFFER(sample.buffer);
assert.sameValue(sample.buffer, buffer);
