// Copyright (C) 2020 Alexey Shvayka. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: sec-object.assign
description: >
  Proxy keys are iterated in order they were provided by "ownKeys" trap.
info: |
  Object.assign ( target, ...sources )

  [...]
  4. For each element nextSource of sources, in ascending index order, do
    a. If nextSource is neither undefined nor null, then
      [...]
      ii. Let keys be ? from.[[OwnPropertyKeys]]().
      iii. For each element nextKey of keys in List order, do
        1. Let desc be ? from.[[GetOwnProperty]](nextKey).

  [[OwnPropertyKeys]] ( )

  [...]
  7. Let trapResultArray be ? Call(trap, handler, « target »).
  8. Let trapResult be ? CreateListFromArrayLike(trapResultArray, « String, Symbol »).
  [...]
  23. Return trapResult.
features: [Proxy, Symbol]
includes: [compareArray.js]
---*/

var getOwnKeys = [];
var ownKeysResult = [Symbol(), "foo", "0"];
var proxy = new Proxy({}, {
  getOwnPropertyDescriptor: function(_target, key) {
    getOwnKeys.push(key);
  },
  ownKeys: function() {
    return ownKeysResult;
  },
});

Object.assign({}, proxy);
assert.compareArray(getOwnKeys, ownKeysResult);
