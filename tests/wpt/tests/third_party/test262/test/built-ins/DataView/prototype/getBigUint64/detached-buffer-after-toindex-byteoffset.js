// Copyright (C) 2017 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-dataview.prototype.getbiguint64
description: >
  Detached buffer is only checked after ToIndex(requestIndex)
includes: [detachArrayBuffer.js]
features: [DataView, ArrayBuffer, BigInt, arrow-function]
---*/

var buffer = new ArrayBuffer(8);
var sample = new DataView(buffer, 0);

$DETACHBUFFER(buffer);

assert.throws(RangeError, () => sample.getBigUint64(Infinity),
  "DataView access at index Infinity should throw");

assert.throws(RangeError, () => sample.getBigUint64(-1),
  "DataView access at index -1 should throw");
