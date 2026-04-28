// Copyright (C) 2023 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: sec-arraybuffer.prototype.transfertofixedlength
description: Transfering from a fixed-size ArrayBuffer into a zero-length ArrayBuffer
features: [arraybuffer-transfer]
---*/

// NOTE: This file is a copy of "from-fixed-to-zero.js" with the resizable
// ArrayBuffer parts removed, so it can run in implementations which don't yet
// support the "resizable-arraybuffer" feature.

var source = new ArrayBuffer(4);

var sourceArray = new Uint8Array(source);
sourceArray[0] = 1;
sourceArray[1] = 2;
sourceArray[2] = 3;
sourceArray[3] = 4;

var dest = source.transferToFixedLength(0);

assert.sameValue(source.byteLength, 0, 'source.byteLength');
assert.throws(TypeError, function() {
  source.slice();
});

assert.sameValue(dest.byteLength, 0, 'dest.byteLength');
