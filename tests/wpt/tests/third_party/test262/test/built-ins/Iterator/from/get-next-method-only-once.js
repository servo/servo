// Copyright (C) 2023 Michael Ficarra. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: sec-iterator.from
description: >
  Gets the next method from the underlying iterator only once
info: |
  Iterator.from ( O )

  2. Let iteratorRecord be ? GetIteratorFlattenable(O).

features: [iterator-helpers]
flags: []
---*/
let nextGets = 0;
let nextCalls = 0;

class CountingIterator {
  get next() {
    ++nextGets;
    let iter = (function* () {
      for (let i = 1; i < 5; ++i) {
        yield i;
      }
    })();
    return function () {
      ++nextCalls;
      return iter.next();
    };
  }
}

let iterator = new CountingIterator();

assert.sameValue(nextGets, 0, 'The value of `nextGets` is 0');
assert.sameValue(nextCalls, 0, 'The value of `nextCalls` is 0');

iterator = Iterator.from(iterator);

assert.sameValue(nextGets, 1, 'The value of `nextGets` is 1');
assert.sameValue(nextCalls, 0, 'The value of `nextCalls` is 0');

iterator.toArray();

assert.sameValue(nextGets, 1, 'The value of `nextGets` is 1');
assert.sameValue(nextCalls, 5, 'The value of `nextCalls` is 5');
