// Copyright (C) 2020 Rick Waldron. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-atomics.compareexchange
description: >
  Atomics.compareExchange throws when operating on incompatible TypedArrays
includes: [testTypedArray.js]
features: [ArrayBuffer, Atomics, TypedArray]
---*/
testWithNonAtomicsFriendlyTypedArrayConstructors((TA, makeCtorArg) => {
  const buffer = makeCtorArg(4);
  const view = new TA(buffer);

  assert.throws(TypeError, function() {
    Atomics.compareExchange(view, 0, 0, 0);
  }, `Atomics.compareExchange(new ${TA.name}(buffer), 0, 0, 0) throws TypeError`);
}, ["arraybuffer"]);
