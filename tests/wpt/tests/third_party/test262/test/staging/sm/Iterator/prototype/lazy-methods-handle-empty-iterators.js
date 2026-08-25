// Copyright (C) 2024 Mozilla Corporation. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: pending
description: |
  Lazy %Iterator.prototype% methods handle empty iterators.
info: |
  Iterator Helpers proposal 2.1.5
features:
  - iterator-helpers
---*/

//
//
class EmptyIterator extends Iterator {
  next() { 
    return {done: true};
  }
}

const emptyIterator1 = new EmptyIterator();
const emptyIterator2 = [].values();

const methods = [
  iter => iter.map(x => x),
  iter => iter.filter(x => x),
  iter => iter.take(1),
  iter => iter.drop(0),
  iter => iter.flatMap(x => [x]),
];

for (const method of methods) {
  for (const iterator of [emptyIterator1, emptyIterator2]) {
    const result = method(iterator).next();
    assert.sameValue(result.done, true);
    assert.sameValue(result.value, undefined);
  }
}

