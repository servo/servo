// Copyright (C) 2026 Saúl Ibarra Corretgé. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-iterator.concat
description: >
  The underlying iterator is sometimes unable to be closed (has no return method)
features: [iterator-sequencing]
---*/

// GetMethod returns undefined for a missing return property and for a null one,
// and IteratorClose then leaves the iterator alone and evaluates to the return
// completion it was given.
function concatWithIterator(testIterator) {
  return Iterator.concat({
    [Symbol.iterator]() {
      return testIterator;
    }
  });
}

function next() {
  return {
    done: false,
    value: 1,
  };
}

// No return property at all.
const iteratorWithoutReturn = concatWithIterator({ next });
iteratorWithoutReturn.next();

const resultWithoutReturn = iteratorWithoutReturn.return();

assert.sameValue(resultWithoutReturn.done, true);
assert.sameValue(resultWithoutReturn.value, undefined);

// An undefined return property.
const iteratorWithUndefinedReturn = concatWithIterator({ next, return: undefined });
iteratorWithUndefinedReturn.next();

const resultWithUndefinedReturn = iteratorWithUndefinedReturn.return();

assert.sameValue(resultWithUndefinedReturn.done, true);
assert.sameValue(resultWithUndefinedReturn.value, undefined);

// A null return property.
const iteratorWithNullReturn = concatWithIterator({ next, return: null });
iteratorWithNullReturn.next();

const resultWithNullReturn = iteratorWithNullReturn.return();

assert.sameValue(resultWithNullReturn.done, true);
assert.sameValue(resultWithNullReturn.value, undefined);
