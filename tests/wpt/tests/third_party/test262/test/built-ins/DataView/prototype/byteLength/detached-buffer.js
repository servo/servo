// Copyright (C) 2016 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-get-dataview.prototype.bytelength
description: Throws a TypeError if the instance has a detached buffer
info: |
  get DataView.prototype.byteLength
  ...
  Let buffer be the value of O's [[ViewedArrayBuffer]] internal slot.
  If IsDetachedBuffer(buffer) is true, throw a TypeError exception.
  ...
includes: [detachArrayBuffer.js]
---*/

let buffer = new ArrayBuffer(1);
let dv = new DataView(buffer, 0);

$DETACHBUFFER(buffer);

assert.throws(TypeError, () => {
  dv.byteLength;
});
