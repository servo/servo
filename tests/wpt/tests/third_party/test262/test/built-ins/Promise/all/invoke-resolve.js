// Copyright (C) 2015 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
description: >
    Invocation of the constructor's `resolve` method
es6id: 25.4.4.1
info: |
    11. Let result be PerformPromiseAll(iteratorRecord, C, promiseCapability).

    [...]

    25.4.4.1.1 Runtime Semantics: PerformPromiseAll

    [...]
    6. Repeat
        [...]
        i. Let nextPromise be Invoke(constructor, "resolve", «nextValue»).
---*/

var p1 = new Promise(function() {});
var p2 = new Promise(function() {});
var p3 = new Promise(function() {});
var resolve = Promise.resolve;
var callCount = 0;
var current = p1;
var next = p2;
var afterNext = p3;

Promise.resolve = function(nextValue) {
  assert.sameValue(
    nextValue, current, '`resolve` invoked with next iterated value'
  );
  assert.sameValue(
    arguments.length, 1, '`resolve` invoked with a single argument'
  );
  assert.sameValue(this, Promise, '`this` value is the constructor');

  current = next;
  next = afterNext;
  afterNext = null;

  callCount += 1;

  return resolve.apply(Promise, arguments);
};

Promise.all([p1, p2, p3]);

assert.sameValue(
  callCount, 3, '`resolve` invoked once for each iterated value'
);
