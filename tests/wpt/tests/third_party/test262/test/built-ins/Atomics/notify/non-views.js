// Copyright (C) 2017 Mozilla Corporation.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-atomics.notify
description: >
  Test Atomics.notify on view values other than TypedArrays
includes: [testAtomics.js]
features: [ArrayBuffer, Atomics, DataView, SharedArrayBuffer, Symbol, TypedArray]
---*/

testWithAtomicsNonViewValues(function(nonView) {
  assert.throws(TypeError, function() {
    Atomics.notify(nonView, 0, 0);
  }); // Even with count == 0
});
