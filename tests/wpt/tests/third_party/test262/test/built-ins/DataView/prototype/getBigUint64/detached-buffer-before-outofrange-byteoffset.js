// Copyright (C) 2017 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-dataview.prototype.getbiguint64
description: >
  Detached buffer is checked before out of range byteOffset's value
includes: [detachArrayBuffer.js]
features: [DataView, ArrayBuffer, BigInt, arrow-function]
---*/

var sample;
var buffer = new ArrayBuffer(12);

sample = new DataView(buffer, 0);

$DETACHBUFFER(buffer);

assert.throws(TypeError, () => sample.getBigUint64(13),
  "detached DataView access should throw");
