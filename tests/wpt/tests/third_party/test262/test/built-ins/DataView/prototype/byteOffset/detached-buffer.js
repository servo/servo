// Copyright (C) 2016 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-get-dataview.prototype.byteoffset
description: Returns 0 if the instance has a detached buffer
info: |
  get DataView.prototype.byteOffset

  5. If IsViewOutOfBounds(viewRecord) is true, return +0𝔽.

  IsViewOutOfBounds ( viewRecord )

  3. If IsDetachedBuffer(view.[[ViewedArrayBuffer]]) is true, then
    b. Return true.
includes: [detachArrayBuffer.js]
---*/

var buffer = new ArrayBuffer(128);
var sample = new DataView(buffer, 8, 1);

$DETACHBUFFER(buffer);

assert.sameValue(sample.byteOffset, 0);
