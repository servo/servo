// Copyright (C) 2020 Rick Waldron. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: sec-integer-indexed-exotic-objects-delete-p
description: >
  Use OrdinaryDelete if key is a Symbol
info: |
  [[Delete]] (P)

  ...
  Assert: IsPropertyKey(P) is true.
  Assert: O is an Integer-Indexed exotic object.
  If Type(P) is String, then
    ...
  Return ? OrdinaryDelete(O, P).

includes: [testTypedArray.js]
features: [align-detached-buffer-semantics-with-web-reality, BigInt, Symbol, TypedArray]
---*/

testWithBigIntTypedArrayConstructors(function(TA, makeCtorArg) {
  let sample = new TA(makeCtorArg(1));
  let s = Symbol("1");

  assert.sameValue(delete sample[s], true, 'The value of `delete sample[s]` is true');
  assert.sameValue(Reflect.has(sample, s), false, 'Reflect.has(sample, s) must return false');

  sample[s] = "";
  assert.sameValue(delete sample[s], true, 'The value of `delete sample[s]` is true');
  assert.sameValue(Reflect.has(sample, s), false, 'Reflect.has(sample, s) must return false');
});
