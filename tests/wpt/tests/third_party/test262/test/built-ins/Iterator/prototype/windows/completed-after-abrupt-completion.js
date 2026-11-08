// Copyright (C) 2026 Saúl Ibarra Corretgé. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: sec-iterator.prototype.windows
description: >
  The iterator helper is completed after an abrupt completion
features: [iterator-chunking, class]
---*/

// The closure of an iterator helper is the body of a generator, so an abrupt
// completion from it completes the generator: a later next() returns an
// undefined, done result without stepping the underlying iterator again, and a
// later return() is not forwarded to it either.
let throwingIteratorNextCalls = 0;
let throwingIteratorReturnCalls = 0;

class ThrowingIterator extends Iterator {
  next() {
    ++throwingIteratorNextCalls;
    throw new Test262Error();
  }
  return() {
    ++throwingIteratorReturnCalls;
    return {};
  }
}

const iteratorWithThrowingNext = new ThrowingIterator().windows(1);

assert.throws(Test262Error, function () {
  iteratorWithThrowingNext.next();
});

assert.sameValue(throwingIteratorNextCalls, 1);

const resultAfterThrow = iteratorWithThrowingNext.next();

assert.sameValue(resultAfterThrow.done, true, 'the iterator helper is completed');
assert.sameValue(resultAfterThrow.value, undefined, 'the iterator helper is completed');
assert.sameValue(throwingIteratorNextCalls, 1, 'the underlying iterator is not stepped again');

const resultAfterReturn = iteratorWithThrowingNext.return();

assert.sameValue(resultAfterReturn.done, true, 'the iterator helper is completed');
assert.sameValue(resultAfterReturn.value, undefined, 'the iterator helper is completed');
assert.sameValue(throwingIteratorReturnCalls, 0, 'return is not forwarded to the underlying iterator');
