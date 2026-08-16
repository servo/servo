// Copyright (C) 2026 Jordan Harband. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-set-error.prototype.stack
description: >
  When a Proxy receiver's defineProperty trap returns false during the
  CreateDataPropertyOrThrow path, or its set trap returns false during the
  Set-with-Throw=true path, the setter throws TypeError.
info: |
  set Error.prototype.stack

  [...]
  4. Perform ? SetterThatIgnoresPrototypeProperties(this value, %Error.prototype%, "stack", v).

  SetterThatIgnoresPrototypeProperties ( this, home, p, v )

  [...]
  3. Let desc be ? this.[[GetOwnProperty]](p).
  4. If desc is undefined, then
    a. Perform ? CreateDataPropertyOrThrow(this, p, v).
  5. Else,
    a. Perform ? Set(this, p, v, true).

  CreateDataPropertyOrThrow ( O, P, V )

  [...]
  3. Let success be ? CreateDataProperty(O, P, V).
  4. If success is false, throw a TypeError exception.

  Set ( O, P, V, Throw )

  [...]
  3. Let success be ? O.[[Set]](P, V, O).
  4. If success is false and Throw is true, throw a TypeError exception.
features: [error-stack-accessor, Proxy]
---*/

var set = Object.getOwnPropertyDescriptor(Error.prototype, 'stack').set;

// (a) defineProperty trap returns false (no own stack: CreateDataPropertyOrThrow path).
var pA = new Proxy({}, {
  defineProperty: function () {
    return false;
  },
});
assert.throws(TypeError, function () {
  set.call(pA, 'v');
}, 'defineProperty returns false');

// (b) set trap returns false (own stack present: Set with Throw=true path).
var pB = new Proxy({ stack: 'old' }, {
  set: function () {
    return false;
  },
});
assert.throws(TypeError, function () {
  set.call(pB, 'v');
}, 'set trap returns false');
