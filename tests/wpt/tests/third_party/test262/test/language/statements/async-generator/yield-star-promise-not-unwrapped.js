// Copyright (C) 2022 Kevin Gibbons. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-generator-function-definitions-runtime-semantics-evaluation
description: >
  `yield*` in an async generator does not await promises returned by a manually implemented async iterator.
flags: [async]
features: [async-iteration]
---*/

var innerPromise = Promise.resolve("unwrapped value");

var asyncIter = {
  [Symbol.asyncIterator]() {
    return this;
  },
  next() {
    return {
      done: false,
      value: innerPromise,
    };
  },
  get return() {
    throw new Test262Error(".return should not be accessed");
  },
  get throw() {
    throw new Test262Error(".throw should not be accessed");
  },
};

async function* f() {
  yield* asyncIter;
}

f()
  .next()
  .then(v => {
    assert.sameValue(v.value, innerPromise, "yield* should not unwrap promises from manually-implemented async iterators");
  })
  .then($DONE, $DONE)
