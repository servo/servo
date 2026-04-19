// Copyright (C) 2020 Rick Waldron. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-atomics.sub
description: >
  Atomics.sub throws when operating on non-sharable integer TypedArrays
includes: [testTypedArray.js]
features: [ArrayBuffer, Atomics, TypedArray]
---*/
testWithNonAtomicsFriendlyTypedArrayConstructors(TA => {
  const buffer = new ArrayBuffer(16);
  const view = new TA(buffer);

  assert.throws(TypeError, function() {
    Atomics.sub(view, 0, 1);
  }, `Atomics.sub(new ${TA.name}(buffer), 0, 1) throws TypeError`);
}, null, ["passthrough"]);
