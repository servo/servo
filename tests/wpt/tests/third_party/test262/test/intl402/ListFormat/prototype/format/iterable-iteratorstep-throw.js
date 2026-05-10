// Copyright 2019 Google Inc.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-Intl.ListFormat.prototype.format
description: >
    Checks the behavior of Abstract Operation StringListFromIterable
    called by Intl.ListFormat.prototype.format() while iteratorStep throws error.
info: |
    StringListFromIterable
    1. If iterable is undefined, then
      a. Return a new empty List.
    2. Let iteratorRecord be ? GetIterator(iterable).
    3. Let list be a new empty List.
    4. Let next be true.
    5. Repeat, while next is not false
      a. Set next to ? IteratorStep(iteratorRecord).
features: [Intl.ListFormat]
---*/
function CustomError() {}

let lf = new Intl.ListFormat();
// Test the failure case.
let iterator_step_throw = {
  [Symbol.iterator]() {
    return this;
  },
  count: 0,
  next() {
    this.count++;
    if (this.count == 3) {
      throw new CustomError();
    }
    if (this.count < 5) {
      return {done: false, value: String(this.count)};
    }
    return {done:true}
  }
};
assert.throws(CustomError,
    ()=> {lf.format(iterator_step_throw)});
assert.sameValue(iterator_step_throw.count, 3);
