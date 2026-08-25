// Copyright (C) 2022 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-array.fromasync
description: If thisArg is an object, it's bound to mapfn as the this-value
info: |
  6. If _mapping_ is *true*, then
    a. Let _mappedValue_ be Call(_mapfn_, _thisArg_, « _nextValue_, 𝔽(_k_) »).
flags: [async]
includes: [asyncHelpers.js]
features: [Array.fromAsync]
---*/

asyncTest(async () => {
  const myThisValue = {};

  await Array.fromAsync([1, 2, 3], async function () {
    assert.sameValue(this, myThisValue, "thisArg should be bound as the this-value of mapfn");
  }, myThisValue);
});
