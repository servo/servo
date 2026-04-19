// Copyright 2019 Google Inc.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-Intl.ListFormat.prototype.format
description: >
    Checks the behavior of Abstract Operation StringListFromIterable
    called by Intl.ListFormat.prototype.formatToParts() while iteratorValue throws error.
info: |
    StringListFromIterable
    1. If iterable is undefined, then
      a. Return a new empty List.
    2. Let iteratorRecord be ? GetIterator(iterable).
    3. Let list be a new empty List.
    4. Let next be true.
    5. Repeat, while next is not false
      a. Set next to ? IteratorStep(iteratorRecord).
      b. If next is not false, then
        i. Let nextValue be ? IteratorValue(next).
features: [Intl.ListFormat]
---*/

function CustomError() {}

let lf = new Intl.ListFormat();
// Test the failure case.
let iterator_value_throw = {
  [Symbol.iterator]() {
    return this;
  },
  count: 0,
  next() {
    this.count++;
    if (this.count == 3) {
      return {done: false, get value() { throw new CustomError() }};
    }
    if (this.count < 5) {
      return {done: false, value: String(this.count)};
    }
    return {done:true}
  }
};
assert.throws(CustomError,
    ()=> {lf.formatToParts(iterator_value_throw)});
assert.sameValue(iterator_value_throw.count, 3);
