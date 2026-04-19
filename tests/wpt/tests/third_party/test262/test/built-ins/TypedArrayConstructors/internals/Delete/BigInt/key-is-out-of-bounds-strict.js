// Copyright (C) 2020 Rick Waldron. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: sec-integer-indexed-exotic-objects-delete-p
description: >
  Return true if IsValidIntegerIndex(O, numericIndex) is false.
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
  Return ? OrdinaryDelete(O, P).

flags: [onlyStrict]
includes: [testTypedArray.js]
features: [align-detached-buffer-semantics-with-web-reality, BigInt, TypedArray]
---*/

testWithBigIntTypedArrayConstructors(function(TA, makeCtorArg) {
  let proto = TypedArray.prototype;
  let descriptorGetterThrows = {
    configurable: true,
    get() {
      throw new Test262Error("OrdinaryGet was called!");
    }
  };
  Object.defineProperties(proto, {
    ["-1"]: descriptorGetterThrows,
    ["1"]: descriptorGetterThrows,
  });

  let sample = new TA(makeCtorArg(1));
  assert.sameValue(delete sample["-1"], true, 'The value of `delete sample["-1"]` is true');
  assert.sameValue(delete sample[-1], true, 'The value of `delete sample[-1]` is true');

  assert.throws(TypeError, () => {
    delete sample["0"];
  });

  assert.throws(TypeError, () => {
    delete sample[0];
  });

  assert.sameValue(delete sample["1"], true, 'The value of `delete sample["1"]` is true');
  assert.sameValue(delete sample[1], true, 'The value of `delete sample[1]` is true');
});
