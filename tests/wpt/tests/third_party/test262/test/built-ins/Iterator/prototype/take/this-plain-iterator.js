// Copyright (C) 2023 Michael Ficarra. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: sec-iteratorprototype.take
description: >
  Iterator.prototype.take supports a this value that does not inherit from Iterator.prototype but implements the iterator protocol
info: |
  %Iterator.prototype%.take ( limit )

  7. Let iterated be ? GetIteratorDirect(this value).

features: [iterator-helpers]
flags: []
---*/
let iter = {
  get next() {
    let count = 3;
    return function () {
      --count;
      return count >= 0 ? { done: false, value: count } : { done: true, value: undefined };
    };
  },
};

let takeIter = Iterator.prototype.take.call(iter, 1);

let { done, value } = takeIter.next();

assert.sameValue(done, false);
assert.sameValue(value, 2);
