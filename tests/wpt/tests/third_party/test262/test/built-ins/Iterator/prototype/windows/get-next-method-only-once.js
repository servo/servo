// Copyright (C) 2026 Michael Ficarra. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: sec-iterator.prototype.windows
description: >
  Gets the next method from the underlying iterator only once
info: |
  Iterator.prototype.windows ( windowSize [ , undersized ] )

  7. Set iterated to ? GetIteratorDirect(O).

features: [iterator-chunking, generators, class]
---*/
let nextGets = 0;
let nextCalls = 0;

class CountingIterator extends Iterator {
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

assert.sameValue(nextGets, 0);
assert.sameValue(nextCalls, 0);

for (const value of iterator.windows(2));

assert.sameValue(nextGets, 1);
assert.sameValue(nextCalls, 5);
