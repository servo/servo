// Copyright (C) 2024 Mozilla Corporation. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
features:
  - iterator-helpers
description: |
  pending
esid: pending
---*/

class TestIterator extends Iterator {
  next(value = "next value") {
    assert.sameValue(arguments.length, 0);
    return {done: false, value};
  }
}

const methods = [
  iter => iter.map(x => x),
  iter => iter.filter(x => true),
  iter => iter.take(2),
  iter => iter.drop(0),
];

for (const outerMethod of methods) {
  for (const innerMethod of methods) {
    const iterator = new TestIterator();
    const iteratorChain = outerMethod(innerMethod(iterator));
    iteratorChain.next();
    let result = iteratorChain.next('last value');
    assert.sameValue(result.done, false);
    assert.sameValue(result.value, 'next value');
  }
}

