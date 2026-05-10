// Copyright (C) 2017 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-dataview.prototype.getbiguint64
description: >
  Throws a TypeError if buffer is detached
includes: [detachArrayBuffer.js]
features: [DataView, ArrayBuffer, BigInt, arrow-function]
---*/

var buffer = new ArrayBuffer(1);
var sample = new DataView(buffer, 0);

$DETACHBUFFER(buffer);
assert.throws(TypeError, () => sample.getBigUint64(0),
  "detached DataView access should throw");
