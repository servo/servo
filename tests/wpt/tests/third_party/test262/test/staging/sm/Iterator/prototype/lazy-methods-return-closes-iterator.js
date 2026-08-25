// Copyright (C) 2024 Mozilla Corporation. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: pending
description: |
  Calling `.return()` on a lazy %Iterator.prototype% method closes the source iterator.
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

  closed = false;
  return(value) {
    this.closed = true;
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

for (const method of methods) {
  const iter = new TestIterator();
  const iterHelper = method(iter);
  iterHelper.next();

  assert.sameValue(iter.closed, false);
  const result = iterHelper.return("ignored");
  assert.sameValue(iter.closed, true);
  assert.sameValue(result.done, true);
  assert.sameValue(result.value, undefined);
}

for (const method of methods) {
  const iter = new TestIterator();
  const iterHelper = method(iter);

  assert.sameValue(iter.closed, false);
  const result = iterHelper.return("ignored");
  assert.sameValue(iter.closed, true);
  assert.sameValue(result.done, true);
  assert.sameValue(result.value, undefined);
}

