// Copyright (C) 2016 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: sec-get-arraybuffer.prototype.bytelength
description: Return value from [[ByteLength]] internal slot
info: |
  24.1.4.1 get ArrayBuffer.prototype.byteLength

  ...
  5. Let length be the value of O's [[ArrayBufferByteLength]] internal slot.
  6. Return length.
---*/

var ab1 = new ArrayBuffer(0);
assert.sameValue(ab1.byteLength, 0);

var ab2 = new ArrayBuffer(42);
assert.sameValue(ab2.byteLength, 42);
