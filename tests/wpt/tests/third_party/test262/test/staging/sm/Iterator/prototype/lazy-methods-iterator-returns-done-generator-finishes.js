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
  next() {
    return {done: true, value: 'value'};
  }
}

const methods = [
  iter => iter.map(x => x),
  iter => iter.filter(x => true),
  iter => iter.take(1),
  iter => iter.drop(0),
  iter => iter.flatMap(x => [x]),
];

for (const method of methods) {
  const iterator = method(new TestIterator());
  const result = iterator.next();
  assert.sameValue(result.done, true);
  assert.sameValue(result.value, undefined);
}

