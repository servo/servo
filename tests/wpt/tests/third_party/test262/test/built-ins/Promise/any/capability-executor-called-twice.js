// Copyright (C) 2019 Sergey Rubanov. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-promise.any
description: >
  Throws a TypeError if capabilities executor already called with non-undefined values.
info: |
  Promise.any ( iterable )

  ...
  2. Let promiseCapability be ? NewPromiseCapability(C).
  ...

  GetCapabilitiesExecutor Functions

  ...
  4. If promiseCapability.[[Resolve]] is not undefined, throw a TypeError exception.
  5. If promiseCapability.[[Reject]] is not undefined, throw a TypeError exception.
  6. Set promiseCapability.[[Resolve]] to resolve.
  7. Set promiseCapability.[[Reject]] to reject.
  ...
features: [Promise.any]
---*/

var checkPoint = '';
function fn1(executor) {
  checkPoint += 'a';
  executor();
  checkPoint += 'b';
  executor(function() {}, function() {});
  checkPoint += 'c';
}
fn1.resolve = function() {
  throw new Test262Error();
};
Promise.any.call(fn1, []);
assert.sameValue(checkPoint, 'abc', 'executor initially called with no arguments');

checkPoint = '';
function fn2(executor) {
  checkPoint += 'a';
  executor(undefined, undefined);
  checkPoint += 'b';
  executor(function() {}, function() {});
  checkPoint += 'c';
}
fn2.resolve = function() {
  throw new Test262Error();
};
Promise.any.call(fn2, []);
assert.sameValue(checkPoint, 'abc', 'executor initially called with (undefined, undefined)');

checkPoint = '';
function fn3(executor) {
  checkPoint += 'a';
  executor(undefined, function() {});
  checkPoint += 'b';
  executor(function() {}, function() {});
  checkPoint += 'c';
}
Object.defineProperty(fn3, 'resolve', {
  get() {
    throw new Test262Error();
  }
});
assert.throws(TypeError, function() {
  Promise.any.call(fn3, []);
}, 'executor initially called with (undefined, function)');
assert.sameValue(checkPoint, 'ab', 'executor initially called with (undefined, function)');

checkPoint = '';
function fn4(executor) {
  checkPoint += 'a';
  executor(function() {}, undefined);
  checkPoint += 'b';
  executor(function() {}, function() {});
  checkPoint += 'c';
}
Object.defineProperty(fn4, 'resolve', {
  get() {
    throw new Test262Error();
  }
});
assert.throws(TypeError, function() {
  Promise.any.call(fn4, []);
}, 'executor initially called with (function, undefined)');
assert.sameValue(checkPoint, 'ab', 'executor initially called with (function, undefined)');

checkPoint = '';
function fn5(executor) {
  checkPoint += 'a';
  executor('invalid value', 123);
  checkPoint += 'b';
  executor(function() {}, function() {});
  checkPoint += 'c';
}
Object.defineProperty(fn5, 'resolve', {
  get() {
    throw new Test262Error();
  }
});
assert.throws(TypeError, function() {
  Promise.any.call(fn5, []);
}, 'executor initially called with (String, Number)');
assert.sameValue(checkPoint, 'ab', 'executor initially called with (String, Number)');
