// Copyright (C) 2019 Leo Balter. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
description: >
  Invocation of the instance's `then` method
esid: sec-promise.allsettled
info: |
  6. Let result be PerformPromiseAllSettled(iteratorRecord, C, promiseCapability).
  7. If result is an abrupt completion, then
    a. If iteratorRecord.[[Done]] is false, set result to IteratorClose(iteratorRecord, result).
    b. IfAbruptRejectPromise(result, promiseCapability).

  Runtime Semantics: PerformPromiseAllSettled

  z. Perform ? Invoke(nextPromise, "then", « resolveElement, rejectElement »).
features: [Promise.allSettled]
---*/

var p1 = new Promise(function() {});
var p2 = new Promise(function() {});
var p3 = new Promise(function() {});
var callCount = 0;
var currentThis = p1;
var nextThis = p2;
var afterNextThis = p3;

p1.then = p2.then = p3.then = function(a, b) {
  assert.sameValue(typeof a, 'function', 'type of first argument');
  assert.sameValue(
    a.length,
    1,
    'The length property of a promise resolve function is 1.'
  );
  assert.sameValue(typeof b, 'function', 'type of second argument');
  assert.sameValue(
    b.length,
    1,
    'The length property of a promise reject function is 1.'
  );
  assert.sameValue(arguments.length, 2, '`then` invoked with two arguments');
  assert.sameValue(this, currentThis, '`this` value');

  currentThis = nextThis;
  nextThis = afterNextThis;
  afterNextThis = null;

  callCount += 1;
};

Promise.allSettled([p1, p2, p3]);

assert.sameValue(callCount, 3, '`then` invoked once for every iterated value');
