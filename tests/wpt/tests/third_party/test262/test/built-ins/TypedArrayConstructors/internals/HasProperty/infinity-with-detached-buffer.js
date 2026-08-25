// Copyright (C) 2017 André Bargull. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-integer-indexed-exotic-objects-hasproperty-p
description: >
  "Infinity" is a canonical numeric string, test with access on detached buffer.
info: |
  9.4.5.2 [[HasProperty]]( P )
  ...
  3. If Type(P) is String, then
    a. Let numericIndex be ! CanonicalNumericIndexString(P).
    b. If numericIndex is not undefined, then
      i. Let buffer be O.[[ViewedArrayBuffer]].
      ii. If IsDetachedBuffer(buffer) is true, return false.
      ...

  7.1.16 CanonicalNumericIndexString ( argument )
    ...
    3. Let n be ! ToNumber(argument).
    4. If SameValue(! ToString(n), argument) is false, return undefined.
    5. Return n.

flags: [noStrict]
includes: [testTypedArray.js, detachArrayBuffer.js]
features: [align-detached-buffer-semantics-with-web-reality, TypedArray]
---*/

testWithTypedArrayConstructors(function(TA) {
  let counter = 0;
  let n = {
    valueOf() {
      counter++;
      return 9;
    }
  };

  assert.sameValue(counter, 0, 'The value of `counter` is 0');

  let ta = new TA([n]);

  assert.sameValue(counter, 1, 'The value of `counter` is 1');

  $DETACHBUFFER(ta.buffer);

  with (ta) {
    Infinity;
    assert.sameValue(counter, 1, 'The value of `counter` is 1');
  }
}, null, ["passthrough"]);
