// Copyright (C) 2026 Saúl Ibarra Corretgé. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: sec-iterator.prototype.windows
description: >
  Underlying iterator is not closed when stepping it throws
features: [iterator-chunking, class]
---*/

// An abrupt completion from stepping the underlying iterator is propagated as
// is, so the iterator is never closed in this file: only an abrupt completion
// from the callback goes through IfAbruptCloseIterator.
let returnCalls = 0;

class TestIterator extends Iterator {
  return() {
    ++returnCalls;
    return {};
  }
}

class ThrowingNextIterator extends TestIterator {
  next() {
    throw new Test262Error();
  }
}

class ThrowingDoneIterator extends TestIterator {
  next() {
    return {
      get done() {
        throw new Test262Error();
      },
      value: 1,
    };
  }
}

class ThrowingValueIterator extends TestIterator {
  next() {
    return {
      done: false,
      get value() {
        throw new Test262Error();
      },
    };
  }
}

class NonObjectIterator extends TestIterator {
  next() {
    return null;
  }
}

// Underlying iterator has a throwing next method
const iteratorWithThrowingNext = new ThrowingNextIterator().windows(1);

assert.throws(Test262Error, function () {
  iteratorWithThrowingNext.next();
});

assert.sameValue(returnCalls, 0, 'return is not called on the underlying iterator');

// Underlying iterator next returns an object with a throwing done getter
const iteratorWithThrowingDoneGetter = new ThrowingDoneIterator().windows(1);

assert.throws(Test262Error, function () {
  iteratorWithThrowingDoneGetter.next();
});

assert.sameValue(returnCalls, 0, 'return is not called on the underlying iterator');

// Underlying iterator next returns an object with a throwing value getter
const iteratorWithThrowingValueGetter = new ThrowingValueIterator().windows(1);

assert.throws(Test262Error, function () {
  iteratorWithThrowingValueGetter.next();
});

assert.sameValue(returnCalls, 0, 'return is not called on the underlying iterator');

// Underlying iterator next returns a non-object
const iteratorWithNonObjectResult = new NonObjectIterator().windows(1);

assert.throws(TypeError, function () {
  iteratorWithNonObjectResult.next();
});

assert.sameValue(returnCalls, 0, 'return is not called on the underlying iterator');
