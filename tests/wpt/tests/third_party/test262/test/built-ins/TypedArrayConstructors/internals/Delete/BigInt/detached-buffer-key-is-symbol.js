// Copyright (C) 2020 Rick Waldron. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: sec-integer-indexed-exotic-objects-delete-p
description: >
  Calls OrdinaryDelete when key is a Symbol.
info: |
  [[Delete]] (P)

  ...
  Assert: IsPropertyKey(P) is true.
  Assert: O is an Integer-Indexed exotic object.
  If Type(P) is String, then
    ...
  Return ? OrdinaryDelete(O, P).
includes: [testTypedArray.js, detachArrayBuffer.js]
features: [align-detached-buffer-semantics-with-web-reality, Symbol, TypedArray]
---*/

testWithTypedArrayConstructors(function(TA) {
  let sample = new TA(1);
  $DETACHBUFFER(sample.buffer);

  let s = Symbol("1");

  sample[s] = 1;
  assert.sameValue(delete sample[s], true, 'The value of `delete sample[s]` is true');
}, null, ["passthrough"]);
