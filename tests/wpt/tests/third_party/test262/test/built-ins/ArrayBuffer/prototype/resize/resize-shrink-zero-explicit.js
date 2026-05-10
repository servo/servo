// Copyright (C) 2021 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: sec-arraybuffer.prototype.resize
description: Behavior when attempting to shrink a resizable array buffer to zero explicitly
info: |
  ArrayBuffer.prototype.resize ( newLength )

  1. Let O be the this value.
  2. Perform ? RequireInternalSlot(O, [[ArrayBufferMaxByteLength]]).
  3. If IsSharedArrayBuffer(O) is true, throw a TypeError exception.
  4. If IsDetachedBuffer(O) is true, throw a TypeError exception.
  5. Let newByteLength be ? ToIntegerOrInfinity(newLength).
  6. If newByteLength < 0 or newByteLength > O.[[ArrayBufferMaxByteLength]],
     throw a RangeError exception.
  7. Let hostHandled be ? HostResizeArrayBuffer(O, newByteLength).
  [...]

  HostResizeArrayBuffer ( buffer, newByteLength )

  The implementation of HostResizeArrayBuffer must conform to the following
  requirements:

  - The abstract operation does not detach buffer.
  - The abstract operation may complete normally or abruptly.
  - If the abstract operation completes normally with handled,
    buffer.[[ArrayBufferByteLength]] is newByteLength.
  - The return value is either handled or unhandled.
features: [resizable-arraybuffer]
---*/

var ab = new ArrayBuffer(4, {maxByteLength: 4});
var caught = false;
var result;

// If the host chooses to throw as allowed by the specification, the observed
// behavior will be identical to the case where `ArrayBuffer.prototype.resize`
// has not been implemented. The following assertion prevents this test from
// passing in runtimes which have not implemented the method.
assert.sameValue(typeof ab.resize, 'function');

try {
  result = ab.resize(0);
} catch (_) {
  caught = true;
}

try {
  ab.slice();
} catch (_) {
  throw new Test262Error('The ArrayBuffer under test was detached');
}

// One of the following three conditions must be met:
//
// - HostResizeArrayBuffer returns an abrupt completion
// - HostResizeArrayBuffer handles the resize operation and conforms to the
//   invarient regarding [[ArrayBufferByteLength]]
// - HostResizeArrayBuffer does not handle the resize operation, and the
//   `resize` method updates [[ArrayBufferByteLength]]
//
// The final two conditions are indistinguishable.
assert(caught || ab.byteLength === 0, 'byteLength');

// One of the following three conditions must be met:
//
// - HostResizeArrayBuffer returns an abrupt completion
// - HostResizeArrayBuffer handles the resize operation, and the `resize`
//   method returns early
// - HostResizeArrayBuffer does not handle the resize operation, and the
//   `resize` method executes its final steps
//
// All three conditions have the same effect on the value of `result`.
assert.sameValue(result, undefined, 'normal completion value');

// The contents of the ArrayBuffer are not guaranteed by the host-defined
// abstract operation, so they are not asserted in this test.
