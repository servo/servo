// Copyright (C) 2021 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: sec-arraybuffer.prototype.transfer
description: Transfering from a fixed-size ArrayBuffer into a smaller ArrayBuffer
info: |
  ArrayBuffer.prototype.transfer ( [ newLength ] )

  1. Let O be the this value.
  2. Perform ? RequireInternalSlot(O, [[ArrayBufferData]]).
  3. If IsSharedArrayBuffer(O) is true, throw a TypeError exception.
  4. If IsDetachedBuffer(O) is true, throw a TypeError exception.
  5. If newLength is undefined, let newByteLength be
     O.[[ArrayBufferByteLength]].
  6. Else, let newByteLength be ? ToIntegerOrInfinity(newLength).
  7. Let new be ? Construct(%ArrayBuffer%, « 𝔽(newByteLength) »).
  8. NOTE: This method returns a fixed-length ArrayBuffer.
  9. Let copyLength be min(newByteLength, O.[[ArrayBufferByteLength]]).
  10. Let fromBlock be O.[[ArrayBufferData]].
  11. Let toBlock be new.[[ArrayBufferData]].
  12. Perform CopyDataBlockBytes(toBlock, 0, fromBlock, 0, copyLength).
  13. NOTE: Neither creation of the new Data Block nor copying from the old
      Data Block are observable. Implementations reserve the right to implement
      this method as a zero-copy move or a realloc.
  14. Perform ! DetachArrayBuffer(O).
  15. Return new.
features: [arraybuffer-transfer]
---*/

// NOTE: This file is a copy of "from-fixed-to-smaller.js" with the resizable
// ArrayBuffer parts removed, so it can run in implementations which don't yet
// support the "resizable-arraybuffer" feature.

var source = new ArrayBuffer(4);

var sourceArray = new Uint8Array(source);
sourceArray[0] = 1;
sourceArray[1] = 2;
sourceArray[2] = 3;
sourceArray[3] = 4;

var dest = source.transfer(3);

assert.sameValue(source.byteLength, 0, 'source.byteLength');
assert.throws(TypeError, function() {
  source.slice();
});

assert.sameValue(dest.byteLength, 3, 'dest.byteLength');

var destArray = new Uint8Array(dest);

assert.sameValue(destArray[0], 1, 'destArray[0]');
assert.sameValue(destArray[1], 2, 'destArray[1]');
assert.sameValue(destArray[2], 3, 'destArray[2]');
