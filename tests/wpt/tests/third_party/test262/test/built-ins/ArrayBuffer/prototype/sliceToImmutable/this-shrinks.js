// Copyright (C) 2025 Richard Gibson. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-arraybuffer.prototype.slicetoimmutable
description: Calculates bounds from original length if receiver shrinks.
info: |
  ArrayBuffer.prototype.sliceToImmutable ( start, end )
  ...
  5. Let len be O.[[ArrayBufferByteLength]].
  6. Let bounds be ? ResolveBounds(len, start, end).
  7. Let first be bounds.[[From]].
  8. Let final be bounds.[[To]].
  9. Let newLen be max(final - first, 0).
  ...
  12. Let fromBuf be O.[[ArrayBufferData]].
  13. Let currentLen be O.[[ArrayBufferByteLength]].
  14. If currentLen < final, throw a RangeError exception.
  15. Let newBuffer be ? AllocateImmutableArrayBuffer(%ArrayBuffer%, newLen, fromBuf, first, newLen).
  16. Return newBuffer.

  ResolveBounds ( len, start, end )
  1. Let relativeStart be ? ToIntegerOrInfinity(start).
  2. If relativeStart = -∞, let from be 0.
  3. Else if relativeStart < 0, let from be max(len + relativeStart, 0).
  4. Else, let from be min(relativeStart, len).
  5. If end is undefined, let relativeEnd be len; else let relativeEnd be ? ToIntegerOrInfinity(end).
  6. If relativeEnd = -∞, let to be 0.
  7. Else if relativeEnd < 0, let to be max(len + relativeEnd, 0).
  8. Else, let to be min(relativeEnd, len).
  9. Return the Record { [[From]]: from, [[To]]: to }.
features: [resizable-arraybuffer, immutable-arraybuffer]
includes: [compareArray.js, detachArrayBuffer.js]
---*/

var source = new ArrayBuffer(10, { maxByteLength: 10 });
var view = new Uint8Array(source);
for (var i = 0; i < 10; i++) view[i] = i + 1;

var startResizeTo = 9;
var endResizeTo = 8;
var start = {
  valueOf() {
    source.resize(startResizeTo);
    return -7;
  }
};
var end = {
  valueOf() {
    source.resize(endResizeTo);
    return -4;
  }
};
var dest = source.sliceToImmutable(start, end);
assert.compareArray(new Uint8Array(dest), [4, 5, 6]);
assert.sameValue(source.byteLength, 8);

source.resize(10);
endResizeTo = 5;
assert.throws(RangeError, function() {
  source.sliceToImmutable(start, end);
}, "resize below resolved end");

source.resize(10);
end = {
  valueOf() {
    source.resize(5);
    $DETACHBUFFER(source);
    return 6;
  }
};
assert.throws(TypeError, function() {
  source.sliceToImmutable(0, end);
}, "Must verify non-detachment before final bounds check");
