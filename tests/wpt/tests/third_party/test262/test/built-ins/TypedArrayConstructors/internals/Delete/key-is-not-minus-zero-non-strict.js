// Copyright (C) 2020 Rick Waldron. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: sec-integer-indexed-exotic-objects-delete-p
description: >
  Return true if key is a CanonicalNumericIndex and IsValidIntegerIndex(O, numericIndex) is false.
info: |
  [[Delete]] (P)

  ...
  Assert: IsPropertyKey(P) is true.
  Assert: O is an Integer-Indexed exotic object.
  If Type(P) is String, then
    Let numericIndex be ! CanonicalNumericIndexString(P).
    If numericIndex is not undefined, then
      If IsDetachedBuffer(O.[[ViewedArrayBuffer]]) is true, return true.
      If ! IsValidIntegerIndex(O, numericIndex) is false, return true.
      Return false.

  ...

  IntegerIndexedElementGet ( O, index )

  ...
  If ! IsValidIntegerIndex(O, index) is false, return undefined.
  ...
flags: [noStrict]
includes: [testTypedArray.js]
features: [align-detached-buffer-semantics-with-web-reality, TypedArray]
---*/

testWithTypedArrayConstructors(function(TA, makeCtorArg) {
  let proto = TypedArray.prototype;
  Object.defineProperty(proto, "-0", {
    configurable: true,
    get() {
      throw new Test262Error("OrdinaryGet was called!");
    }
  });
  let sample = new TA(makeCtorArg(1));

  assert.sameValue(delete sample["-0"], true, 'The value of `delete sample["-0"]` is true');
  assert.sameValue(delete sample[-0], false, 'The value of `delete sample[-0]` is false');
}, null, ["passthrough"]);
