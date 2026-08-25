// Copyright (C) 2017 André Bargull. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
description: >
    Including isConstructor.js will expose one function:

      isConstructor

includes: [isConstructor.js]
features: [generators, Reflect.construct]
---*/

assert.sameValue(typeof isConstructor, "function");

assert.throws(Test262Error, () => isConstructor(), "no argument");
assert.throws(Test262Error, () => isConstructor(undefined), "undefined");
assert.throws(Test262Error, () => isConstructor(null), "null");
assert.throws(Test262Error, () => isConstructor(123), "number");
assert.throws(Test262Error, () => isConstructor(true), "boolean - true");
assert.throws(Test262Error, () => isConstructor(false), "boolean - false");
assert.throws(Test262Error, () => isConstructor("string"), "string");

assert.throws(Test262Error, () => isConstructor({}), "Object instance");
assert.throws(Test262Error, () => isConstructor([]), "Array instance");

assert.sameValue(isConstructor(function(){}), true);
assert.sameValue(isConstructor(function*(){}), false);
assert.sameValue(isConstructor(() => {}), false);

assert.sameValue(isConstructor(Array), true);
assert.sameValue(isConstructor(Array.prototype.map), false);
