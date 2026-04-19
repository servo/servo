// Copyright (C) 2023 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-array.fromasync
description: >
  A synchronous mapping function is applied to each item yielded by an
  asynchronous iterable.
info: |
  3.j.ii.6. If _mapping_ is *true*, then
    a. Let _mappedValue_ be Call(_mapfn_, _thisArg_, « _nextValue_, 𝔽(_k_) »).
    ...
  ...
  8. Let _defineStatus_ be CreateDataPropertyOrThrow(_A_, _Pk_, _mappedValue_).
flags: [async]
includes: [asyncHelpers.js, compareArray.js]
features: [Array.fromAsync]
---*/

async function* asyncGen() {
  for (let i = 0; i < 4; i++) {
    yield Promise.resolve(i * 2);
  }
}

function syncMap(val, ix) {
  return val * ix;
}

asyncTest(async () => {
  const result = await Array.fromAsync({ [Symbol.asyncIterator]: asyncGen }, syncMap);
  assert.compareArray(result, [0, 2, 8, 18], "sync mapfn should be applied to async iterable");
});
