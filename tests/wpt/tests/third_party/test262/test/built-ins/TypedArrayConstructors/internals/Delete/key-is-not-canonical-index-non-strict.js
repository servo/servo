// Copyright (C) 2020 Rick Waldron. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: sec-integer-indexed-exotic-objects-delete-p
description: >
  Return true if key is not a CanonicalNumericIndex.
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
flags: [noStrict]
includes: [testTypedArray.js]
features: [align-detached-buffer-semantics-with-web-reality, TypedArray]
---*/

testWithTypedArrayConstructors(function(TA) {
  var keys = [
    "1.0",
    "+1",
    "1000000000000000000000",
    "0.0000001"
  ];

  keys.forEach((key) => {
    var sample = new TA(); // <- intentionally empty

    assert.sameValue(
      delete sample[key], true,
      'The value of `delete sample[key]` is true'
    );

    TypedArray.prototype[key] = key;

    assert.sameValue(
      delete sample[key],
      true,
      'The value of `delete sample[key]` is true'
    );

    sample[key] = key;
    assert.sameValue(
      delete sample[key], true,
      'The value of `delete sample[key]` is true'
    );

    Object.defineProperty(sample, key, {
      get() { return key; }
    });

    assert.sameValue(
      delete sample[key], false,
      'The value of `delete sample[key]` is false'
    );

    delete TypedArray.prototype[key];
  });
}, null, ["passthrough"]);
