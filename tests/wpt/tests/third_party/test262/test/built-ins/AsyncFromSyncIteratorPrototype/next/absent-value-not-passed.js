// Copyright (C) 2020 Alexey Shvayka. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-%asyncfromsynciteratorprototype%.next
description: >
  `next` method does not pass absent `value`.
info: |
  %AsyncFromSyncIteratorPrototype%.next ( value )

  [...]
  5. If value is present, then
    [...]
  6. Else,
    a. Let result be IteratorNext(syncIteratorRecord).
  [...]
flags: [async]
features: [async-iteration]
includes: [asyncHelpers.js]
---*/

var nextArgumentsLength;
var syncIterator = {
  [Symbol.iterator]() {
    return this;
  },
  next() {
    nextArgumentsLength = arguments.length;
    return {done: true};
  },
};

asyncTest(async function () {
  for await (let _ of syncIterator);

  assert.sameValue(nextArgumentsLength, 0);
});
