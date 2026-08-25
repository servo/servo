// Copyright (C) 2022 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-array.fromasync
description: >
  A TypeError is thrown if the mapfn argument to Array.fromAsync is not callable
info: |
  3.a. If _mapfn_ is *undefined*, let _mapping_ be *false*.
    b. Else,
      i. If IsCallable(_mapfn_) is *false*, throw a *TypeError* exception.
flags: [async]
includes: [asyncHelpers.js]
features: [Array.fromAsync, BigInt, Symbol]
---*/

asyncTest(async () => {
  await assert.throwsAsync(TypeError, () => Array.fromAsync([], null), "null mapfn");
  await assert.throwsAsync(TypeError, () => Array.fromAsync([], {}), "non-callable object mapfn");
  await assert.throwsAsync(TypeError, () => Array.fromAsync([], "String"), "string mapfn");
  await assert.throwsAsync(TypeError, () => Array.fromAsync([], true), "boolean mapfn");
  await assert.throwsAsync(TypeError, () => Array.fromAsync([], 3.1416), "number mapfn");
  await assert.throwsAsync(TypeError, () => Array.fromAsync([], 42n), "bigint mapfn");
  await assert.throwsAsync(TypeError, () => Array.fromAsync([], Symbol()), "symbol mapfn");
});
