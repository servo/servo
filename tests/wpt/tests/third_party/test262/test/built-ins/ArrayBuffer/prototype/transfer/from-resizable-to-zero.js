// Copyright (C) 2021 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: sec-arraybuffer.prototype.transfer
description: Transfering from a resizable ArrayBuffer into a zero-length ArrayBuffer
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
features: [resizable-arraybuffer, arraybuffer-transfer]
---*/

var source = new ArrayBuffer(4, { maxByteLength: 8 });

var sourceArray = new Uint8Array(source);
sourceArray[0] = 1;
sourceArray[1] = 2;
sourceArray[2] = 3;
sourceArray[3] = 4;

var dest = source.transfer(0);

assert.sameValue(source.byteLength, 0, 'source.byteLength');
assert.throws(TypeError, function() {
  source.slice();
});

assert.sameValue(dest.resizable, true, 'dest.resizable');
assert.sameValue(dest.byteLength, 0, 'dest.byteLength');
assert.sameValue(dest.maxByteLength, 8, 'dest.maxByteLength');
