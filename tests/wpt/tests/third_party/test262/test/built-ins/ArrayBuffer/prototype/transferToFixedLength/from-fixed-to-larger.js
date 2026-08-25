// Copyright (C) 2023 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: sec-arraybuffer.prototype.transfertofixedlength
description: Transfering from a fixed-size ArrayBuffer into a larger ArrayBuffer
features: [resizable-arraybuffer, arraybuffer-transfer]
---*/

var source = new ArrayBuffer(4);

var sourceArray = new Uint8Array(source);
sourceArray[0] = 1;
sourceArray[1] = 2;
sourceArray[2] = 3;
sourceArray[3] = 4;

var dest = source.transferToFixedLength(5);

assert.sameValue(source.byteLength, 0, 'source.byteLength');
assert.throws(TypeError, function() {
  source.slice();
});

assert.sameValue(dest.resizable, false, 'dest.resizable');
assert.sameValue(dest.byteLength, 5, 'dest.byteLength');
assert.sameValue(dest.maxByteLength, 5, 'dest.maxByteLength');

var destArray = new Uint8Array(dest);

assert.sameValue(destArray[0], 1, 'destArray[0]');
assert.sameValue(destArray[1], 2, 'destArray[1]');
assert.sameValue(destArray[2], 3, 'destArray[2]');
assert.sameValue(destArray[3], 4, 'destArray[3]');
assert.sameValue(destArray[4], 0, 'destArray[4]');
