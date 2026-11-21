// Copyright (C) 2026 Saúl Ibarra Corretgé. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-iterator.concat
description: >
  The return value of Iterator.concat is an Iterator Helper
features: [iterator-sequencing, iterator-helpers]
---*/

// Iterator.concat builds its result with %IteratorHelperPrototype%, so it shares
// its prototype, and with it its next and return methods and its @@toStringTag,
// with the iterator helpers returned by %Iterator.prototype%.map and friends.
const iteratorHelperPrototype = Object.getPrototypeOf([].values().map(v => v));

const iteratorWithoutArguments = Iterator.concat();

assert.sameValue(
  Object.getPrototypeOf(iteratorWithoutArguments),
  iteratorHelperPrototype,
  'Object.getPrototypeOf(Iterator.concat()) must return %IteratorHelperPrototype%'
);
assert.sameValue(iteratorWithoutArguments.next, iteratorHelperPrototype.next);
assert.sameValue(iteratorWithoutArguments.return, iteratorHelperPrototype.return);
assert.sameValue(Object.prototype.toString.call(iteratorWithoutArguments), '[object Iterator Helper]');

const iterable = {
  [Symbol.iterator]() {
    return {
      next() {
        return {
          done: true,
          value: undefined,
        };
      },
    };
  }
};

const iteratorWithIterable = Iterator.concat(iterable);

assert.sameValue(
  Object.getPrototypeOf(iteratorWithIterable),
  iteratorHelperPrototype,
  'Object.getPrototypeOf(Iterator.concat(iterable)) must return %IteratorHelperPrototype%'
);
assert.sameValue(iteratorWithIterable.next, iteratorHelperPrototype.next);
assert.sameValue(iteratorWithIterable.return, iteratorHelperPrototype.return);
assert.sameValue(Object.prototype.toString.call(iteratorWithIterable), '[object Iterator Helper]');
