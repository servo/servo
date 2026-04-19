// Copyright 2020 Rick Waldron. All rights reserved.
// This code is governed by the license found in the LICENSE file.

/*---
esid:  sec-object.assign
description: >
  Assignment to readonly property of target must throw a TypeError exception
info: |
  Let to be ? ToObject(target).
  If only one argument was passed, return to.
  For each element nextSource of sources, do
    If nextSource is neither undefined nor null, then
      Let from be ! ToObject(nextSource).
      Let keys be ? from.[[OwnPropertyKeys]]().
      For each element nextKey of keys, do
        Let desc be ? from.[[GetOwnProperty]](nextKey).
        If desc is not undefined and desc.[[Enumerable]] is true, then
          Let propValue be ? Get(from, nextKey).
          Perform ? Set(to, nextKey, propValue, true).

  Set ( O, P, V, Throw ) sec-set-o-p-v-throw

  Let success be ? O.[[Set]](P, V, O).
  If success is false and Throw is true, throw a TypeError exception.
---*/

assert.throws(TypeError, () => {
  Object.assign('a', [1]);
});
