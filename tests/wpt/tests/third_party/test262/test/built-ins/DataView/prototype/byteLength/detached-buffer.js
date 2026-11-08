// Copyright (C) 2016 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-get-dataview.prototype.bytelength
description: Returns 0 if the instance has a detached buffer
info: |
  get DataView.prototype.byteLength

  5. If IsViewOutOfBounds(viewRecord) is true, return +0𝔽.

  IsViewOutOfBounds ( viewRecord )

  3. If IsDetachedBuffer(view.[[ViewedArrayBuffer]]) is true, then
    b. Return true.
includes: [detachArrayBuffer.js]
---*/

let buffer = new ArrayBuffer(1);
let dv = new DataView(buffer, 0);

$DETACHBUFFER(buffer);

assert.sameValue(dv.byteLength, 0);
