// Copyright (C) 2024 Mozilla Corporation. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: pending
description: |
  Lazy Iterator Helper methods return new iterator result objects.
info: |
  Iterator Helpers proposal 2.1.5
features:
  - iterator-helpers
---*/

//
//
const iterResult = {done: false, value: 1, testProperty: 'test'};
class TestIterator extends Iterator {
  next() {
    return iterResult;
  }
}

const methods = [
  iter => iter.map(x => x),
  iter => iter.filter(x => true),
  iter => iter.take(1),
  iter => iter.drop(0),
  iter => iter.flatMap(x => [x]),
];

// Call `return` before stepping the iterator:
for (const method of methods) {
  const iter = new TestIterator();
  const iterHelper = method(iter);
  const result = iterHelper.next();
  assert.sameValue(result == iterResult, false);
  assert.sameValue(result.testProperty, undefined);
}

