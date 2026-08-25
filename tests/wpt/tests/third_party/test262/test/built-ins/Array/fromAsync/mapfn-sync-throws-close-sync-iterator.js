// Copyright (C) 2023 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-array.fromasync
description: >
  The iterator of a synchronous iterable is closed when the synchronous mapping
  function throws.
info: |
  3.j.ii.6. If _mapping_ is *true*, then
    a. Let _mappedValue_ be Call(_mapfn_, _thisArg_, « _nextValue_, 𝔽(_k_) »).
    b. IfAbruptCloseAsyncIterator(_mappedValue_, _iteratorRecord_).
    ...
flags: [async]
includes: [asyncHelpers.js]
features: [Array.fromAsync]
---*/

let closed = false;
const iterator = {
  next() {
    return { value: 1, done: false };
  },
  return() {
    closed = true;
    return { done: true };
  },
  [Symbol.iterator]() {
    return this;
  }
}

asyncTest(async () => {
  await assert.throwsAsync(Error, () => Array.fromAsync(iterator, (val) => {
    assert.sameValue(val, 1, "mapfn receives value from iterator");
    throw new Error("mapfn throws");
  }), "sync mapfn throwing should cause fromAsync to reject");
  assert(closed, "sync mapfn throwing should close iterator")
});
