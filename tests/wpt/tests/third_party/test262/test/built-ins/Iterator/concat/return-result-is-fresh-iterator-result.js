// Copyright (C) 2026 Saúl Ibarra Corretgé. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-iterator.concat
description: >
  return returns a fresh iterator result object, whatever the underlying iterator's return method returned
features: [iterator-sequencing]
---*/

const innerIterResult = {
  done: false,
  value: 'inner',
};

const testIterator = {
  next() {
    return {
      done: false,
      value: 1,
    };
  },
  return() {
    return innerIterResult;
  },
};

const iterable = {
  [Symbol.iterator]() {
    return testIterator;
  }
};

const iterator = Iterator.concat(iterable);
iterator.next();

const iterResult = iterator.return();

assert.notSameValue(iterResult, innerIterResult);
assert.sameValue(iterResult.done, true);
assert.sameValue(iterResult.value, undefined);

// The argument of return is not observable in the result either.
const iteratorCalledWithArgument = Iterator.concat(iterable);
iteratorCalledWithArgument.next();

const iterResultWithArgument = iteratorCalledWithArgument.return('argument');

assert.sameValue(iterResultWithArgument.done, true);
assert.sameValue(iterResultWithArgument.value, undefined);
