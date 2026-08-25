// Copyright (C) 2019 Alexey Shvayka. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: sec-catchfinallyfunctions
description: >
  thrower is anonymous built-in function with length of 1 that throws reason.
info: |
  Catch Finally Functions

  ...
  8. Let thrower be equivalent to a function that throws reason.
  9. Return ? Invoke(promise, "then", « thrower »).

  The "length" property of a Catch Finally function is 1.
features: [Promise.prototype.finally, Reflect.construct, class, arrow-function]
includes: [isConstructor.js]
flags: [async]
---*/

class MyError extends Error {}

var myError = new MyError();
Promise.reject(myError)
  .finally(function() {})
  .then(function(value) {
    $DONE('Expected promise to be rejected, got fulfilled with ' + value);
  }, function(reason) {
    if (reason === myError) {
      $DONE();
    } else {
      $DONE(reason);
    }
  });

var calls = 0;
var expected = [
  { length: 0, name: '' },
  { length: 1, name: '' }
];

var then = Promise.prototype.then;
Promise.prototype.then = function(resolve, reject) {

  assert.sameValue(
    resolve.length,
    expected[calls].length,
    'The value of resolve.length is expected to equal the value of expected[calls].length'
  );
  assert.sameValue(
    resolve.name,
    expected[calls].name,
    'The value of resolve.name is expected to equal the value of expected[calls].name'
  );
  if (calls === 0) {
    assert.throws(MyError, resolve, '`resolve()` throws `MyError`');
    assert.sameValue(arguments.length, 1, '`then` invoked with one argument');
  } else {
    assert.sameValue(isConstructor(reject), false, 'isConstructor(reject) must return false');
    assert.throws(TypeError, () => {
      new reject();
    });
    assert.sameValue(arguments.length, 2, '`then` invoked with two arguments');
  }

  calls += 1;

  return then.call(this, resolve, reject);
};
