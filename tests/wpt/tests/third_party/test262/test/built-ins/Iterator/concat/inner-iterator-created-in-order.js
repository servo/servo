// Copyright (C) 2024 André Bargull. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-iterator.concat
description: >
  Inner iterators created in order
info: |
  Iterator.concat ( ...items )

  ...
  3. Let closure be a new Abstract Closure with no parameters that captures iterables and performs the following steps when called:
    a. For each Record iterable of iterables, do
      i. Let iter be ? Call(iterable.[[OpenMethod]], iterable.[[Iterable]]).
      ...
      v. Repeat, while innerAlive is true,
        ...
features: [iterator-sequencing]
includes: [compareArray.js]
---*/

let calledIterator = [];

let iterable1 = {
  [Symbol.iterator]() {
    calledIterator.push("iterable1");
    return [1][Symbol.iterator]();
  }
};

let iterable2 = {
  [Symbol.iterator]() {
    calledIterator.push("iterable2");
    return [2][Symbol.iterator]();
  }
};

let iterator = Iterator.concat(iterable1, iterable2);

assert.compareArray(calledIterator, []);

let iterResult = iterator.next();
assert.sameValue(iterResult.done, false);
assert.sameValue(iterResult.value, 1);

assert.compareArray(calledIterator, ["iterable1"]);

iterResult = iterator.next();
assert.sameValue(iterResult.done, false);
assert.sameValue(iterResult.value, 2);

assert.compareArray(calledIterator, ["iterable1", "iterable2"]);
