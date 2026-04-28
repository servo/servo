// Copyright (C) 2017 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-dataview.prototype.setbigint64
description: >
  Detached buffer is checked before out of range byteOffset's value
includes: [detachArrayBuffer.js]
features: [DataView, ArrayBuffer, BigInt]
---*/

var sample;
var buffer = new ArrayBuffer(12);

sample = new DataView(buffer, 0);

$DETACHBUFFER(buffer);

assert.throws(TypeError, function() {
  sample.setBigInt64(13, 0);
}, "detached DataView access should throw");
