// Copyright (C) 2017 Mozilla Corporation.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-atomics.exchange
description: >
  Test Atomics.exchange on view values other than TypedArrays
includes: [testAtomics.js]
features: [ArrayBuffer, Atomics, DataView, SharedArrayBuffer, Symbol, TypedArray]
---*/

testWithAtomicsNonViewValues(function(view) {
  assert.throws(TypeError, function() {
    Atomics.exchange(view, 0, 0);
  });
});
