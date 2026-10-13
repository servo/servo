// Copyright (C) 2025 Richard Gibson. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-arraybuffer.prototype.slicetoimmutable
description: Throws if receiver is detached by bounds resolution
info: |
  ArrayBuffer.prototype.sliceToImmutable ( start, end )
  ...
  6. Let bounds be ? ResolveBounds(len, start, end).
  ...
  10. NOTE: Side-effects of the above steps may have detached or resized O.
  11. If IsDetachedBuffer(O) is true, throw a TypeError exception.

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
features: [immutable-arraybuffer]
includes: [compareArray.js, detachArrayBuffer.js]
---*/

var fn = ArrayBuffer.prototype.sliceToImmutable;
assert.sameValue(typeof fn, "function", "Method must exist.");

var ab = new ArrayBuffer(8);
var calls = [];
var objStart = {
  valueOf() {
    calls.push("start.valueOf");
    return 0;
  }
};
var objEnd = {
  valueOf() {
    $DETACHBUFFER(ab);
    calls.push("end.valueOf");
    return 1;
  }
};
assert.throws(TypeError, function() {
  ab.sliceToImmutable(objStart, objEnd);
});
assert.compareArray(calls, ["start.valueOf", "end.valueOf"]);
