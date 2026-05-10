// Copyright (C) 2024 Mozilla Corporation. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: pending
description: |
  Lazy %Iterator.prototype% method calls can be interleaved.
info: |
  Iterator Helpers proposal 2.1.5
features:
  - iterator-helpers
---*/

//
//
class TestIterator extends Iterator {
  value = 0;
  next() { 
    return {done: false, value: this.value++};
  }
}

function unwrapResult(result) {
  return result;
}

const methods = [
  iter => iter.map(x => x),
  iter => iter.filter(x => true),
  iter => iter.take(2),
  iter => iter.drop(0),
  iter => iter.flatMap(x => [x]),
];

for (const firstMethod of methods) {
  for (const secondMethod of methods) {
    const iterator = new TestIterator();
    const firstHelper = firstMethod(iterator);
    const secondHelper = secondMethod(iterator);

    let firstResult = unwrapResult(firstHelper.next());
    assert.sameValue(firstResult.done, false);
    assert.sameValue(firstResult.value, 0);

    let secondResult = unwrapResult(secondHelper.next());
    assert.sameValue(secondResult.done, false);
    assert.sameValue(secondResult.value, 1);

    firstResult = unwrapResult(firstHelper.next());
    assert.sameValue(firstResult.done, false);
    assert.sameValue(firstResult.value, 2);

    secondResult = unwrapResult(secondHelper.next());
    assert.sameValue(secondResult.done, false);
    assert.sameValue(secondResult.value, 3);
  }
}

