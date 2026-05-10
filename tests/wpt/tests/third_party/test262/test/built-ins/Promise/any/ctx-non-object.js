// Copyright (C) 2020 Rick Waldron. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
description: >
    Promise.any invoked on a non-object value
esid: sec-promise.any
info: |
  ...
  2. Let promiseCapability be ? NewPromiseCapability(C).

  NewPromiseCapability ( C )

  1. If IsConstructor(C) is false, throw a TypeError exception.

features: [Promise.any, Symbol]
---*/

assert.throws(TypeError, function() {
  Promise.any.call(undefined, []);
});

assert.throws(TypeError, function() {
  Promise.any.call(null, []);
});

assert.throws(TypeError, function() {
  Promise.any.call(86, []);
});

assert.throws(TypeError, function() {
  Promise.any.call('string', []);
});

assert.throws(TypeError, function() {
  Promise.any.call(true, []);
});

assert.throws(TypeError, function() {
  Promise.any.call(Symbol(), []);
});
