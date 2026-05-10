// Copyright (C) 2023 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-array.fromasync
description: >
  The iterator of an asynchronous iterable is closed when the asynchronous
  mapping function throws.
info: |
  3.j.ii.6. If _mapping_ is *true*, then
    a. Let _mappedValue_ be Call(_mapfn_, _thisArg_, « _nextValue_, 𝔽(_k_) »).
    ...
    c. Set _mappedValue_ to Await(_mappedValue_).
    d. IfAbruptCloseAsyncIterator(_mappedValue_, _iteratorRecord_).
flags: [async]
includes: [asyncHelpers.js]
features: [Array.fromAsync]
---*/

let closed = false;
const iterator = {
  next() {
    return Promise.resolve({ value: 1, done: false });
  },
  return() {
    closed = true;
    return Promise.resolve({ done: true });
  },
  [Symbol.asyncIterator]() {
    return this;
  }
}

asyncTest(async () => {
  await assert.throwsAsync(Error, () => Array.fromAsync(iterator, async (val) => {
    assert.sameValue(val, 1, "mapfn receives value from iterator");
    throw new Error("mapfn throws");
  }), "async mapfn rejecting should cause fromAsync to reject");
  assert(closed, "async mapfn rejecting should close iterator")
});
