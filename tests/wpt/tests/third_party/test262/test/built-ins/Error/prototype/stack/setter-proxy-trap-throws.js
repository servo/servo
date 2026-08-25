// Copyright (C) 2026 Jordan Harband. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-set-error.prototype.stack
description: >
  Errors thrown by Proxy traps invoked during the setter propagate out via
  the ? abstract operations in SetterThatIgnoresPrototypeProperties.
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
features: [error-stack-accessor, Proxy]
---*/

var set = Object.getOwnPropertyDescriptor(Error.prototype, 'stack').set;

function Sentinel() {}

// (a) getOwnPropertyDescriptor trap throws: error propagates from step 3.
var pA = new Proxy({}, {
  getOwnPropertyDescriptor: function () {
    throw new Sentinel();
  },
});
assert.throws(Sentinel, function () {
  set.call(pA, 'v');
}, 'getOwnPropertyDescriptor trap throw');

// (b) defineProperty trap throws (no own stack: CreateDataPropertyOrThrow path).
var pB = new Proxy({}, {
  defineProperty: function () {
    throw new Sentinel();
  },
});
assert.throws(Sentinel, function () {
  set.call(pB, 'v');
}, 'defineProperty trap throw');

// (c) set trap throws (own stack present: Set path).
var pC = new Proxy({ stack: 'old' }, {
  set: function () {
    throw new Sentinel();
  },
});
assert.throws(Sentinel, function () {
  set.call(pC, 'v');
}, 'set trap throw');
