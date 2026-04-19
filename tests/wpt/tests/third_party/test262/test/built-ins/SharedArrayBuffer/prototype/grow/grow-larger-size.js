// Copyright (C) 2021 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: sec-sharedarraybuffer.prototype.grow
description: >
  Behavior when attempting to grow a growable array buffer to a larger size
info: |
  SharedArrayBuffer.prototype.grow ( newLength )

  1. Let O be the this value.
  2. Perform ? RequireInternalSlot(O, [[ArrayBufferMaxByteLength]]).
  3. If IsSharedArrayBuffer(O) is false throw a TypeError exception.
  4. Let newByteLength be ? ToIntegerOrInfinity(newLength).
  5. Let hostHandled be ? HostGrowSharedArrayBuffer(O, newByteLength).
  6. If hostHandled is handled, return undefined.
  [...]
features: [SharedArrayBuffer, resizable-arraybuffer]
---*/

var sab = new SharedArrayBuffer(4, {maxByteLength: 5});
var result;

// If the host chooses to throw as allowed by the specification, the observed
// behavior will be identical to the case where
// `SharedArrayBuffer.prototype.grow` has not been implemented. The following
// assertion prevents this test from passing in runtimes which have not
// implemented the method.
assert.sameValue(typeof sab.grow, 'function');

try {
  result = ab.grow(5);
} catch (_) {}

// One of the following three conditions must be met:
//
// - HostGrowSharedArrayBuffer returns an abrupt completion
// - HostGrowSharedArrayBuffer handles the grow operation, and the `grow`
//   method returns early
// - HostGrowSharedArrayBuffer does not handle the grow operation, and the
//   `grow` method executes its final steps
//
// All three conditions have the same effect on the value of `result`.
assert.sameValue(result, undefined, 'normal completion value');

// Neither the length nor the contents of the SharedArrayBuffer are guaranteed
// by the host-defined abstract operation, so they are not asserted in this
// test.
