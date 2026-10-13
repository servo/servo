// Copyright (C) 2018 Rick Waldron.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-atomics.islockfree
description: >
  Atomics.isLockFree returns a boolean that indicates whether
  operations on datum of size will be performed without the agent
  acquiring a lock outside of size bytes.
info: |
  Atomics.isLockFree( size )

  1. Let n be ? ToInteger(size).
  2. Let AR be the Agent Record of the surrounding agent.
  3. If n equals 1, return AR.[[IsLockFree1]].
  4. If n equals 2, return AR.[[IsLockFree2]].
  5. If n equals 4, return true.
  6. If n equals 8, return AR.[[IsLockFree8]].
  7. Return false.

features: [Atomics, BigInt, SharedArrayBuffer, TypedArray]
includes: [testTypedArray.js]
---*/

testWithBigIntTypedArrayConstructors(function(TA) {
  var observed = Atomics.isLockFree(TA.BYTES_PER_ELEMENT);

  assert.sameValue(
    Atomics.isLockFree(TA.BYTES_PER_ELEMENT),
    observed,
    'Atomics.isLockFree(TA.BYTES_PER_ELEMENT) returns the value of `observed` (Atomics.isLockFree(TA.BYTES_PER_ELEMENT))'
  );
}, null, ["passthrough"]);
