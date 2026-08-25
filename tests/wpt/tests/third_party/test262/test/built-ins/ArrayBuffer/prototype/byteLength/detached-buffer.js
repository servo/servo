// Copyright (C) 2016 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: sec-get-arraybuffer.prototype.bytelength
description: Returns 0 if the buffer is detached
info: |
  get ArrayBuffer.prototype.byteLength
  ...
  If IsDetachedBuffer(buffer) is true, return 0.
  ...
includes: [detachArrayBuffer.js]
features: [align-detached-buffer-semantics-with-web-reality]
---*/

var ab = new ArrayBuffer(1);

$DETACHBUFFER(ab);

assert.sameValue(ab.byteLength, 0, 'The value of ab.byteLength is 0');
