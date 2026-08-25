// Copyright (C) 2024 Mozilla Corporation. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: pending
description: |
  Calling `.return()` on a lazy %Iterator.prototype% method multiple times closes the source iterator once.
info: |
  Iterator Helpers proposal 2.1.5
features:
  - iterator-helpers
---*/

//
//
class TestIterator extends Iterator {
  next() { 
    return {done: false, value: 1};
  }

  closeCount = 0;
  return(value) {
    this.closeCount++;
    return {done: true, value};
  }
}

const methods = [
  iter => iter.map(x => x),
  iter => iter.filter(x => x),
  iter => iter.take(1),
  iter => iter.drop(0),
  iter => iter.flatMap(x => [x]),
];

// Call `return` after stepping the iterator once:
for (const method of methods) {
  const iter = new TestIterator();
  const iterHelper = method(iter);
  iterHelper.next();

  assert.sameValue(iter.closeCount, 0);
  iterHelper.return();
  assert.sameValue(iter.closeCount, 1);
  iterHelper.return();
  assert.sameValue(iter.closeCount, 1);
}

// Call `return` before stepping the iterator:
for (const method of methods) {
  const iter = new TestIterator();
  const iterHelper = method(iter);

  assert.sameValue(iter.closeCount, 0);
  iterHelper.return();
  assert.sameValue(iter.closeCount, 1);
  iterHelper.return();
  assert.sameValue(iter.closeCount, 1);
}

