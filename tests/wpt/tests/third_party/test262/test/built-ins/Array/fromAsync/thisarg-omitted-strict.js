// Copyright (C) 2022 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-array.fromasync
description: >
  If thisArg is omitted, mapfn is called with undefined as the this-value in
  strict mode
info: |
  6. If _mapping_ is *true*, then
    a. Let _mappedValue_ be Call(_mapfn_, _thisArg_, « _nextValue_, 𝔽(_k_) »).

  In OrdinaryCallBindThis, _thisArgument_ is always bound as the this-value in
  strict mode (_F_.[[ThisMode]] is ~strict~, where _F_ is the function object.)
flags: [async, onlyStrict]
includes: [asyncHelpers.js]
features: [Array.fromAsync]
---*/

asyncTest(async () => {
  await Array.fromAsync([1, 2, 3], async function () {
    assert.sameValue(
      this,
      undefined,
      "undefined should be bound as the this-value of mapfn when thisArg is omitted"
    );
  });
});
