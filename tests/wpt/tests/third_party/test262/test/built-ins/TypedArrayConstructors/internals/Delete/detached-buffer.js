// Copyright (C) 2020 Rick Waldron. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: sec-integer-indexed-exotic-objects-delete-p
description: >
  Returns true when deleting any property if buffer is detached.
info: |
  [[Delete]] (P)

  ...
  Assert: IsPropertyKey(P) is true.
  Assert: O is an Integer-Indexed exotic object.
  If Type(P) is String, then
    Let numericIndex be ! CanonicalNumericIndexString(P).
    If numericIndex is not undefined, then
      If IsDetachedBuffer(O.[[ViewedArrayBuffer]]) is true, return true.
  ...
includes: [testTypedArray.js, detachArrayBuffer.js]
features: [align-detached-buffer-semantics-with-web-reality, TypedArray]
---*/

testWithTypedArrayConstructors(function(TA) {
  var sample = new TA(1);
  $DETACHBUFFER(sample.buffer);
  assert.sameValue(delete sample[0], true, 'The value of `delete sample[0]` is true');
  assert.sameValue(delete sample["1.1"], true, 'The value of `delete sample["1.1"]` is true');
  assert.sameValue(delete sample["-0"], true, 'The value of `delete sample["-0"]` is true');
  assert.sameValue(delete sample["-1"], true, 'The value of `delete sample["-1"]` is true');
  assert.sameValue(delete sample["1"], true, 'The value of `delete sample["1"]` is true');
  assert.sameValue(delete sample["2"], true, 'The value of `delete sample["2"]` is true');
}, null, ["passthrough"]);
