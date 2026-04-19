// Copyright (C) 2024 André Bargull. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-iterator.concat
description: >
  Arguments are validated in order.
info: |
  Iterator.concat ( ...items )

  1. Let iterables be a new empty List.
  2. For each element item of items, do
    a. If item is not an Object, throw a TypeError exception.
    b. Let method be ? GetMethod(item, %Symbol.iterator%).
    ...
features: [iterator-sequencing]
---*/

let getIterator = 0;

let iterable1 = {
  get [Symbol.iterator]() {
    getIterator++;
    return function() {
      throw new Test262Error();
    };
  }
};

let iterable2 = {
  get [Symbol.iterator]() {
    throw new Test262Error();
  }
};

assert.sameValue(getIterator, 0);

assert.throws(TypeError, function() {
  Iterator.concat(iterable1, null, iterable2);
});

assert.sameValue(getIterator, 1);
