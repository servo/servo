// Copyright (C) 2023 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-array.fromasync
description: >
  The returned value from each invocation of the asynchronous mapping function
  is awaited exactly once.
info: |
  3.j.ii.6. If _mapping_ is *true*, then
    a. Let _mappedValue_ be Call(_mapfn_, _thisArg_, « _nextValue_, 𝔽(_k_) »).
    ...
    c. Set _mappedValue_ to Await(_mappedValue_).
flags: [async]
includes: [asyncHelpers.js, compareArray.js, temporalHelpers.js]
features: [Array.fromAsync]
---*/

const calls = [];
const expected = [
  "call mapping",
  "get thenable_0.then",
  "call thenable_0.then",
  "call mapping",
  "get thenable_1.then",
  "call thenable_1.then",
  "call mapping",
  "get thenable_2.then",
  "call thenable_2.then",
];

function mapping(val, ix) {
  calls.push("call mapping");
  const thenableName = `thenable_${ix}`;
  return TemporalHelpers.propertyBagObserver(calls, {
    then(resolve, reject) {
      calls.push(`call ${thenableName}.then`);
      resolve(val * 2);
    }
  }, thenableName)
}

asyncTest(async () => {
  const result = await Array.fromAsync([1, 2, 3], mapping);
  assert.compareArray(result, [2, 4, 6], "mapping function applied");
  assert.compareArray(calls, expected, "observable operations");
});
