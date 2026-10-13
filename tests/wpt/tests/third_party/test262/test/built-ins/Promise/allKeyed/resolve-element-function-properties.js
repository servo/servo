// Copyright (C) 2026 Danial Asaria (Bloomberg LP). All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-performpromiseallkeyed
description: >
  Promise.allKeyed resolve element functions have the expected function shape
info: |
  PerformPromiseAllKeyed ( variant, promises, ctor, resultCapability, promiseResolve )

  ...
  Let onFulfilled be CreateBuiltinFunction(fulfilledSteps, 1, "", « [[AlreadyCalled]], [[Index]] »).
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
var secondOnFulfilled;
var input = {
  first: {
    then: function(onFulfilled) {
      firstOnFulfilled = onFulfilled;
    }
  },
  second: {
    then: function(onFulfilled) {
      secondOnFulfilled = onFulfilled;
    }
  }
};

Promise.allKeyed.call(Constructor, input);

verifyProperty(firstOnFulfilled, "length", {
  value: 1,
  writable: false,
  enumerable: false,
  configurable: true
});
verifyProperty(firstOnFulfilled, "name", {
  value: "",
  writable: false,
  enumerable: false,
  configurable: true
});
assert.sameValue(Object.getPrototypeOf(firstOnFulfilled), Function.prototype);
assert.sameValue(
  Object.prototype.hasOwnProperty.call(firstOnFulfilled, "prototype"),
  false,
  "resolve element function does not have a prototype property"
);
assert.sameValue(isConstructor(firstOnFulfilled), false, "resolve element function is not a constructor");
assert(Object.isExtensible(firstOnFulfilled), "resolve element function is extensible");
assert.notSameValue(firstOnFulfilled, secondOnFulfilled, "each key gets a distinct resolve element function");
