// Copyright (C) 2026 Saúl Ibarra Corretgé. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-iterator.concat
description: >
  Underlying iterator is not closed when stepping it throws
features: [iterator-sequencing]
---*/

// The only IteratorClose in the closure of Iterator.concat is the one performed
// when the yield completes abruptly. An abrupt completion from stepping the
// iterator of an iterable is propagated as is, so the iterator is never closed
// in this file.
let returnCalls = 0;

function concatWithStep(next) {
  return Iterator.concat({
    [Symbol.iterator]() {
      return {
        next,
        return() {
          ++returnCalls;
          return {};
        },
      };
    }
  });
}

// Underlying iterator has a throwing next method.
const iteratorWithThrowingNext = concatWithStep(function() {
  throw new Test262Error();
});

assert.throws(Test262Error, function() {
  iteratorWithThrowingNext.next();
});

assert.sameValue(returnCalls, 0, 'return is not called on the underlying iterator');

// Underlying iterator next returns an object with a throwing done getter.
const iteratorWithThrowingDoneGetter = concatWithStep(function() {
  return {
    get done() {
      throw new Test262Error();
    },
    value: 1,
  };
});

assert.throws(Test262Error, function() {
  iteratorWithThrowingDoneGetter.next();
});

assert.sameValue(returnCalls, 0, 'return is not called on the underlying iterator');

// Underlying iterator next returns an object with a throwing value getter.
const iteratorWithThrowingValueGetter = concatWithStep(function() {
  return {
    done: false,
    get value() {
      throw new Test262Error();
    },
  };
});

assert.throws(Test262Error, function() {
  iteratorWithThrowingValueGetter.next();
});

assert.sameValue(returnCalls, 0, 'return is not called on the underlying iterator');

// Underlying iterator next returns a non-object.
const iteratorWithNonObjectResult = concatWithStep(function() {
  return null;
});

assert.throws(TypeError, function() {
  iteratorWithNonObjectResult.next();
});

assert.sameValue(returnCalls, 0, 'return is not called on the underlying iterator');
