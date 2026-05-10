// Copyright (C) 2022 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-array.fromasync
description: >
  If thisArg is omitted, mapfn is called with the global object as the
  this-value in sloppy mode
info: |
  6. If _mapping_ is *true*, then
    a. Let _mappedValue_ be Call(_mapfn_, _thisArg_, « _nextValue_, 𝔽(_k_) »).

  OrdinaryCallBindThis, when _F_.[[ThisMode]] is ~global~, where _F_ is the
  function object:
  6. Else,
    a. If _thisArgument_ is *undefined* or *null*, then
      i. Let _globalEnv_ be _calleeRealm_.[[GlobalEnv]].
      ii. Assert: _globalEnv_ is a Global Environment Record.
      iii. Let _thisValue_ be _globalEnv_.[[GlobalThisValue]].
flags: [async, noStrict]
includes: [asyncHelpers.js]
features: [Array.fromAsync]
---*/

asyncTest(async () => {
  await Array.fromAsync([1, 2, 3], async function () {
    assert.sameValue(
      this,
      globalThis,
      "the global should be bound as the this-value of mapfn when thisArg is omitted"
    );
  });
});
