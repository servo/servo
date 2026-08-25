// Copyright (C) 2024 Mozilla Corporation. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: pending
description: |
  Lazy %Iterator.prototype% methods throw if `next` call returns a non-object.
info: |
  Iterator Helpers proposal 2.1.5
features:
  - iterator-helpers
---*/

//
//
class TestIterator extends Iterator {
  next(value) {
    return value;
  }

  closed = false;
  return() {
    this.closed = true;
    return {done: true};
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
  for (const value of [undefined, null, 0, false, '', Symbol('')]) {
    const iterator = new TestIterator();
    assert.sameValue(iterator.closed, false);
    assert.throws(TypeError, () => method(iterator).next(value));
    assert.sameValue(iterator.closed, false);
  }
}

