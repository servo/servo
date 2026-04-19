// Copyright (C) 2023 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-array.fromasync
description: >
  Constructs the this-value once if asyncItems is iterable, twice if not, and
  length and element properties are set correctly on the result
info: |
  3.e. If IsConstructor(_C_) is *true*, then
    i. Let _A_ be ? Construct(_C_).
  ...
  j. If _iteratorRecord_ is not *undefined*, then
    ...
  k. Else,
    ...
    iv. If IsConstructor(_C_) is *true*, then
      1. Let _A_ be ? Construct(_C_, « 𝔽(_len_) »).
includes: [compareArray.js, asyncHelpers.js]
flags: [async]
features: [Array.fromAsync]
---*/

asyncTest(async function () {
  const constructorCalls = [];

  function MyArray(...args) {
    constructorCalls.push(args);
  }

  let result = await Array.fromAsync.call(MyArray, [1, 2]);
  assert(result instanceof MyArray, "result is an instance of the constructor this-value");
  assert.sameValue(result.length, 2, "length is set on result");
  assert.sameValue(result[0], 1, "element 0 is set on result");
  assert.sameValue(result[1], 2, "element 1 is set on result");
  assert.sameValue(constructorCalls.length, 1, "constructor is called once");
  assert.compareArray(constructorCalls[0], [], "constructor is called with no arguments");

  constructorCalls.splice(0);  // reset

  result = await Array.fromAsync.call(MyArray, {
    length: 2,
    0: 1,
    1: 2
  });
  assert(result instanceof MyArray, "result is an instance of the constructor this-value");
  assert.sameValue(result.length, 2, "length is set on result");
  assert.sameValue(result[0], 1, "element 0 is set on result");
  assert.sameValue(result[1], 2, "element 1 is set on result");
  assert.sameValue(constructorCalls.length, 1, "constructor is called once");
  assert.compareArray(constructorCalls[0], [2], "constructor is called with a length argument");
});
