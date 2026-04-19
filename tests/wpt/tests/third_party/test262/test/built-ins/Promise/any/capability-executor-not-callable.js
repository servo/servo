// Copyright (C) 2019 Sergey Rubanov. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-promise.any
description: >
  Throws a TypeError if either resolve or reject capability is not callable.
info: |
  Promise.any ( iterable )

  ...
  2. Let promiseCapability be ? NewPromiseCapability(C).
  ...

  NewPromiseCapability ( C )

  ...
  5. Let executor be ! CreateBuiltinFunction(steps, « [[Capability]] »).
  6. Set executor.[[Capability]] to promiseCapability.
  7. Let promise be ? Construct(C, « executor »).
  8. If IsCallable(promiseCapability.[[Resolve]]) is false, throw a TypeError exception.
  9. If IsCallable(promiseCapability.[[Reject]]) is false, throw a TypeError exception.
  ...
features: [Promise.any]
---*/

var checkPoint = '';
function fn1(executor) {
  checkPoint += 'a';
}
Object.defineProperty(fn1, 'resolve', {
  get() { throw new Test262Error(); }
});
assert.throws(TypeError, function() {
  Promise.any.call(fn1, []);
}, 'executor not called at all');
assert.sameValue(checkPoint, 'a', 'executor not called at all');

checkPoint = '';
function fn2(executor) {
  checkPoint += 'a';
  executor();
  checkPoint += 'b';
}
Object.defineProperty(fn2, 'resolve', {
  get() { throw new Test262Error(); }
});
assert.throws(TypeError, function() {
  Promise.any.call(fn2, []);
}, 'executor called with no arguments');
assert.sameValue(checkPoint, 'ab', 'executor called with no arguments');

checkPoint = '';
function fn3(executor) {
  checkPoint += 'a';
  executor(undefined, undefined);
  checkPoint += 'b';
}
Object.defineProperty(fn3, 'resolve', {
  get() { throw new Test262Error(); }
});
assert.throws(TypeError, function() {
  Promise.any.call(fn3, []);
}, 'executor called with (undefined, undefined)');
assert.sameValue(checkPoint, 'ab', 'executor called with (undefined, undefined)');

checkPoint = '';
function fn4(executor) {
  checkPoint += 'a';
  executor(undefined, function() {});
  checkPoint += 'b';
}
Object.defineProperty(fn4, 'resolve', {
  get() { throw new Test262Error(); }
});
assert.throws(TypeError, function() {
  Promise.any.call(fn4, []);
}, 'executor called with (undefined, function)');
assert.sameValue(checkPoint, 'ab', 'executor called with (undefined, function)');

checkPoint = '';
function fn5(executor) {
  checkPoint += 'a';
  executor(function() {}, undefined);
  checkPoint += 'b';
}
Object.defineProperty(fn5, 'resolve', {
  get() { throw new Test262Error(); }
});
assert.throws(TypeError, function() {
  Promise.any.call(fn5, []);
}, 'executor called with (function, undefined)');
assert.sameValue(checkPoint, 'ab', 'executor called with (function, undefined)');

checkPoint = '';
function fn6(executor) {
  checkPoint += 'a';
  executor(123, 'invalid value');
  checkPoint += 'b';
}
Object.defineProperty(fn6, 'resolve', {
  get() { throw new Test262Error(); }
});
assert.throws(TypeError, function() {
  Promise.any.call(fn6, []);
}, 'executor called with (Number, String)');
assert.sameValue(checkPoint, 'ab', 'executor called with (Number, String)');
