// Copyright 2019 Google Inc.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-Intl.ListFormat.prototype.format
description: >
    Checks the behavior of Abstract Operation StringListFromIterable    called by Intl.ListFormat.prototype.format() while the GetIterator
    throws error.
info: |
    StringListFromIterable
    1. If iterable is undefined, then
      a. Return a new empty List.
    2. Let iteratorRecord be ? GetIterator(iterable).
features: [Intl.ListFormat]
---*/

function CustomError() {}

let lf = new Intl.ListFormat();
// Test the failure case.
let get_iterator_throw_error = {
  [Symbol.iterator]() {
    throw new CustomError();
  }
};
assert.throws(CustomError,
    ()=> {lf.format(get_iterator_throw_error)});
