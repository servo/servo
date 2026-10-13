// Copyright (C) 2025 Richard Gibson. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-arraybuffer.prototype.transfertoimmutable
description: >
  `ab.transferToImmutable(sameOrSmallerByteLength)` detaches `ab` and returns an
  immutable ArrayBuffer with contents equal to the specified-length prefix of
  `ab`
info: |
  ArrayBuffer.prototype.transferToImmutable ( [ newLength ] )
  1. Let O be the this value.
  2. Return ? ArrayBufferCopyAndDetach(O, newLength, ~immutable~).

  ArrayBufferCopyAndDetach ( arrayBuffer, newLength, preserveResizability )
  1. Perform ? RequireInternalSlot(arrayBuffer, [[ArrayBufferData]]).
  2. If IsSharedArrayBuffer(arrayBuffer) is true, throw a TypeError exception.
  3. If newLength is undefined, then
     a. Let newByteLength be arrayBuffer.[[ArrayBufferByteLength]].
  4. Else,
     a. Let newByteLength be ? ToIndex(newLength).
  5. If IsDetachedBuffer(arrayBuffer) is true, throw a TypeError exception.
  6. If IsImmutableBuffer(arrayBuffer) is true, throw a TypeError exception.
  7. If arrayBuffer.[[ArrayBufferDetachKey]] is not undefined, throw a TypeError exception.
  8. Let copyLength be min(newByteLength, arrayBuffer.[[ArrayBufferByteLength]]).
  9. If preserveResizability is immutable, then
     a. Let newBuffer be ? AllocateImmutableArrayBuffer(%ArrayBuffer%, newByteLength, arrayBuffer.[[ArrayBufferData]], 0, copyLength).
  10. Else, ...
  11. Perform ! DetachArrayBuffer(arrayBuffer).
features: [Uint8Array, immutable-arraybuffer]
includes: [compareArray.js]
---*/

var byteLength = 4;
var sourceMakers = [
  function fixed() {
    return new ArrayBuffer(byteLength);
  },
  ArrayBuffer.prototype.resize && function resizable() {
    return new ArrayBuffer(byteLength, { maxByteLength: byteLength * 2 });
  },
  ArrayBuffer.prototype.resize && function shrunk() {
    var ab = new ArrayBuffer(byteLength * 2, { maxByteLength: byteLength * 2 });
    ab.resize(byteLength);
    return ab;
  },
  ArrayBuffer.prototype.resize && function grown() {
    var ab = new ArrayBuffer(byteLength / 2, { maxByteLength: byteLength });
    ab.resize(byteLength);
    return ab;
  }
];

for (var i = 0; i < sourceMakers.length; i++) {
  if (!sourceMakers[i]) continue;
  var name = sourceMakers[i].name;
  for (var j = -1; j <= byteLength; j++) {
    var newLength = j < 0 ? undefined : j;
    var label = name + "(" + byteLength + ").transferToImmutable(" + newLength + ")";
    var source = sourceMakers[i]();
    var sourceView = new Uint8Array(source);
    for (var k = 0; k < byteLength; k++) sourceView[k] = k + 1;
    var expectDestContents = sourceView.slice(0, newLength);

    var dest = source.transferToImmutable(newLength);
    assert.sameValue(source.detached, true, label + " source detached");
    assert.sameValue(dest.immutable, true, label + " is immutable");
    assert.sameValue(dest.resizable, false, label + " is not resizable");
    var resolvedNewLength = newLength === undefined ? byteLength : newLength;
    assert.sameValue(dest.byteLength, resolvedNewLength, label + " byteLength");
    assert.sameValue(dest.maxByteLength, resolvedNewLength, label + " maxByteLength");

    var destView = new Uint8Array(dest);
    assert.compareArray(destView, expectDestContents, label + " contents");
  }
}
