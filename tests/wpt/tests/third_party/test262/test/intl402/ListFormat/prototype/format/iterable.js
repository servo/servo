// Copyright 2019 Google Inc.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-Intl.ListFormat.prototype.format
description: >
    Checks the behavior of Abstract Operation StringListFromIterable
    called by Intl.ListFormat.prototype.format().
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
        ii. If Type(nextValue) is not String, then
          1. Let error be ThrowCompletion(a newly created TypeError object).
          2. Return ? IteratorClose(iteratorRecord, error).
        iii. Append nextValue to the end of the List list.
    6. Return list.
features: [Intl.ListFormat]
---*/

let lf = new Intl.ListFormat();

// Test the success case.
let iterable_of_strings = {
  [Symbol.iterator]() {
    return this;
  },
  count: 0,
  next() {
    this.count++
    if (this.count < 4) {
      return {done: false, value: String(this.count)};
    }
    return {done:true}
  }
};
lf.format(iterable_of_strings);
assert.sameValue(iterable_of_strings.count, 4);
