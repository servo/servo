// Copyright (C) 2016 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-get-dataview.prototype.byteoffset
description: Throws a TypeError if the instance has a detached buffer
info: |
  24.2.4.3 get DataView.prototype.byteOffset

  ...
  5. Let buffer be the value of O's [[ViewedArrayBuffer]] internal slot.
  6. If IsDetachedBuffer(buffer) is true, throw a TypeError exception.
  ...
includes: [detachArrayBuffer.js]
---*/

var buffer = new ArrayBuffer(1);
var sample = new DataView(buffer, 0);

$DETACHBUFFER(buffer);

assert.throws(TypeError, function() {
  sample.byteOffset;
});
