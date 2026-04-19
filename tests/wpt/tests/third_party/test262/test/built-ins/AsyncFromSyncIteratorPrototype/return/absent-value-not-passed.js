// Copyright (C) 2020 Alexey Shvayka. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-%asyncfromsynciteratorprototype%.return
description: >
  `return` method does not pass absent `value`.
info: |
  %AsyncFromSyncIteratorPrototype%.return ( value )

  [...]
  8. If value is present, then
    [...]
  9. Else,
    a. Let result be Call(return, syncIterator).
  [...]
flags: [async]
features: [async-iteration]
includes: [asyncHelpers.js]
---*/

var returnArgumentsLength;
var syncIterator = {
  [Symbol.iterator]() {
    return this;
  },
  next() {
    return {done: false};
  },
  return() {
    returnArgumentsLength = arguments.length;
    return {done: true};
  },
};

asyncTest(async function () {
  for await (let _ of syncIterator) {
    break;
  }

  assert.sameValue(returnArgumentsLength, 0);
});
