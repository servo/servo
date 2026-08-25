// Copyright (C) 2024 André Bargull. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-iterator.concat
description: >
  Gets the iterator method from the input iterables only once.
info: |
  Iterator.concat ( ...items )

  1. Let iterables be a new empty List.
  2. For each element item of items, do
    a. If item is not an Object, throw a TypeError exception.
    b. Let method be ? GetMethod(item, %Symbol.iterator%).
    c. If method is undefined, throw a TypeError exception.
    d. Append the Record { [[OpenMethod]]: method, [[Iterable]]: item } to iterables.
  ...
features: [iterator-sequencing]
includes: [compareArray.js]
---*/

let iteratorGets = 0;
let iteratorCalls = 0;
let array = [1, 2, 3];

class CountingIterable {
  get [Symbol.iterator]() {
    ++iteratorGets;

    return function () {
      ++iteratorCalls;
      return array[Symbol.iterator]();
    };
  }
}

let iterable = new CountingIterable();

assert.sameValue(iteratorGets, 0);
assert.sameValue(iteratorCalls, 0);

let iter = Iterator.concat(iterable);

assert.sameValue(iteratorGets, 1);
assert.sameValue(iteratorCalls, 0);

let result = [...iter];

assert.sameValue(iteratorGets, 1);
assert.sameValue(iteratorCalls, 1);

assert.compareArray(result, array);
