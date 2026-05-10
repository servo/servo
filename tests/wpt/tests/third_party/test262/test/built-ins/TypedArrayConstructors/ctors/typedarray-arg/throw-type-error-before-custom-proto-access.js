// Copyright (C) 2025 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: sec-typedarray
description: >
  The newTarget getter must not be evaluated if argument processing throws an error before AllocateTypedArray is reached
info: |
  23.2.5.1 TypedArray ( ...args )
    ...
    5. If numberOfArgs = 0, then
      a. Return ? AllocateTypedArray(constructorName, NewTarget, proto, 0).
    6. Else,
      a. Let firstArgument be args[0].
      b. If firstArgument is an Object, then
        ...
      c. Else,
        i. Assert: firstArgument is not an Object.
        ii. Let elementLength be ? ToIndex(firstArgument).
        iii. Return ? AllocateTypedArray(constructorName, NewTarget, proto, elementLength).

  7.1.22 ToIndex ( value )
    1. Let integer be ? ToIntegerOrInfinity(value).
    ...

  7.1.5 ToIntegerOrInfinity ( argument )
    1. Let number be ? ToNumber(argument).
    ...

  7.1.4 ToNumber ( argument )
    1. If argument is a Number, return argument.
    2. If argument is either a Symbol or a BigInt, throw a TypeError exception.
    ...

includes: [testTypedArray.js]
features: [Reflect, TypedArray]
---*/

var newTarget = function () {}.bind(null);
Object.defineProperty(newTarget, "prototype", {
  get() {
    throw new Test262Error();
  },
});

testWithTypedArrayConstructors(function (TA) {
  assert.throws(TypeError, function () {
    Reflect.construct(TA, [Symbol()], newTarget);
  });
}, null, ["passthrough"]);
