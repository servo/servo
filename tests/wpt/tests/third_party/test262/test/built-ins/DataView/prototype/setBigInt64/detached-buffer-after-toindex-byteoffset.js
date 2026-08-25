// Copyright (C) 2017 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-dataview.prototype.setbigint64
description: >
  Detached buffer is only checked after ToIndex(requestIndex)
includes: [detachArrayBuffer.js]
features: [DataView, ArrayBuffer, BigInt]
---*/

var buffer = new ArrayBuffer(12);
var sample = new DataView(buffer, 0);

$DETACHBUFFER(buffer);

assert.throws(RangeError, function() {
  sample.setBigInt64(Infinity, 0);
}, "DataView access at index Infinity should throw");

assert.throws(RangeError, function() {
  sample.setBigInt64(-1, 0);
}, "DataView access at index -1 should throw");
