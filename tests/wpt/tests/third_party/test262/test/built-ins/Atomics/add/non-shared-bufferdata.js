// Copyright (C) 2020 Rick Waldron. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-atomics.add
description: >
  Atomics.add will operate on TA when TA.buffer is not a SharedArrayBuffer
includes: [testTypedArray.js]
features: [ArrayBuffer, Atomics, TypedArray]
---*/
testWithAtomicsFriendlyTypedArrayConstructors((TA, makeCtorArg) => {
  const view = new TA(makeCtorArg(4));

  assert.sameValue(Atomics.add(view, 0, 1), 0, 'Atomics.add(view, 0, 1) returns 0');
  assert.sameValue(Atomics.load(view, 0), 1, 'Atomics.load(view, 0) returns 1');
}, ["arraybuffer"], ["immutable"]);
