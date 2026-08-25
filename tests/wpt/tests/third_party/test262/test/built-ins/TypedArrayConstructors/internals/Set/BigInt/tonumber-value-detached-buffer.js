// Copyright (C) 2017 Mozilla Corporation. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-integer-indexed-exotic-objects-set-p-v-receiver
description: >
  Setting a typed array element to a value that, when converted to the typed
  array element type, detaches the typed array's underlying buffer,
  will always return true.
info: |
  9.4.5.5 [[Set]] ( P, V, Receiver)

  ...
  2. If Type(P) is String, then
    a. Let numericIndex be ! CanonicalNumericIndexString(P).
    b. If numericIndex is not undefined, then
      i. Perform ? IntegerIndexedElementSet(O, numericIndex, V).
      ii. Return true.
  ...
includes: [testTypedArray.js, detachArrayBuffer.js]
features: [align-detached-buffer-semantics-with-web-reality, BigInt, Reflect, TypedArray]
---*/

testWithBigIntTypedArrayConstructors(function(TA) {
  let ta = new TA(1);
  let isDetached = false;
  let result = Reflect.set(ta, 0, {
    valueOf() {
      $DETACHBUFFER(ta.buffer);
      isDetached = true;
      return 42n;
    }
  });

  assert.sameValue(result, true);
  assert.sameValue(ta[0], undefined);
  assert.sameValue(isDetached, true);
}, null, ["passthrough"]);
