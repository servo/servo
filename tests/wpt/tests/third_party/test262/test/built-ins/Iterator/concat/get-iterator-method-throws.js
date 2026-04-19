// Copyright (C) 2024 André Bargull. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-iterator.concat
description: >
  Error thrown when retrieving the iterator method.
info: |
  Iterator.concat ( ...items )

  1. Let iterables be a new empty List.
  2. For each element item of items, do
    a. If item is not an Object, throw a TypeError exception.
    b. Let method be ? GetMethod(item, %Symbol.iterator%).
    ...
features: [iterator-sequencing]
---*/

var iterable = {
  get [Symbol.iterator]() {
    throw new Test262Error();
  }
};

assert.throws(Test262Error, function() {
  Iterator.concat(iterable);
});
