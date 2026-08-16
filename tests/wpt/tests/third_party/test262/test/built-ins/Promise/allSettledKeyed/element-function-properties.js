// Copyright (C) 2026 Danial Asaria (Bloomberg LP). All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-performpromiseallkeyed
description: >
  Promise.allSettledKeyed resolve and reject element functions have the expected
  function shape
info: |
  PerformPromiseAllKeyed ( variant, promises, ctor, resultCapability, promiseResolve )

  ...
  Let onFulfilled be CreateBuiltinFunction(fulfilledSteps, 1, "", « [[AlreadyCalled]], [[Index]] »).
  ...
  Let onRejected be CreateBuiltinFunction(rejectedSteps, 1, "", « [[AlreadyCalled]], [[Index]] »).
includes: [propertyHelper.js, isConstructor.js]
features: [await-dictionary, Reflect.construct]
---*/

function Constructor(executor) {
  executor(function() {}, Test262Error.thrower);
}
Constructor.resolve = function(value) {
  return value;
};

var firstOnFulfilled;
var firstOnRejected;
var secondOnFulfilled;
var secondOnRejected;
var input = {
  first: {
    then: function(onFulfilled, onRejected) {
      firstOnFulfilled = onFulfilled;
      firstOnRejected = onRejected;
    }
  },
  second: {
    then: function(onFulfilled, onRejected) {
      secondOnFulfilled = onFulfilled;
      secondOnRejected = onRejected;
    }
  }
};

function verifyElementFunction(fn, name) {
  verifyProperty(fn, "length", {
    value: 1,
    writable: false,
    enumerable: false,
    configurable: true
  });
  verifyProperty(fn, "name", {
    value: "",
    writable: false,
    enumerable: false,
    configurable: true
  });
  assert.sameValue(Object.getPrototypeOf(fn), Function.prototype, name + " prototype");
  assert.sameValue(
    Object.prototype.hasOwnProperty.call(fn, "prototype"),
    false,
    name + " does not have a prototype property"
  );
  assert.sameValue(isConstructor(fn), false, name + " is not a constructor");
  assert(Object.isExtensible(fn), name + " is extensible");
}

Promise.allSettledKeyed.call(Constructor, input);

verifyElementFunction(firstOnFulfilled, "first onFulfilled");
verifyElementFunction(firstOnRejected, "first onRejected");
assert.notSameValue(firstOnFulfilled, firstOnRejected, "resolve and reject functions are distinct");
assert.notSameValue(firstOnFulfilled, secondOnFulfilled, "each key gets a distinct resolve element function");
assert.notSameValue(firstOnRejected, secondOnRejected, "each key gets a distinct reject element function");
