// Copyright (C) 2024 Justin Dorfman. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: sec-addrestrictedfunctionproperties
description: >
  Function.prototype.arguments is an accessor property whose set and get
  functions are both %ThrowTypeError%.
info: |
  2. Let _thrower_ be _realm_.[[Intrinsics]].[[%ThrowTypeError%]].
  3. Perform ! DefinePropertyOrThrow(_F_, *"caller"*, PropertyDescriptor { [[Get]]: _thrower_, [[Set]]: _thrower_, [[Enumerable]]: *false*, [[Configurable]]: *true* }).
  4. Perform ! DefinePropertyOrThrow(_F_, *"arguments"*, PropertyDescriptor { [[Get]]: _thrower_, [[Set]]: _thrower_, [[Enumerable]]: *false*, [[Configurable]]: *true* }).
includes: [propertyHelper.js, wellKnownIntrinsicObjects.js]
---*/

const argumentsDesc = Object.getOwnPropertyDescriptor(Function.prototype, 'arguments');

verifyProperty(
  Function.prototype,
  "arguments",
  { enumerable: false, configurable: true },
  { restore: true }
);

assert.sameValue(typeof argumentsDesc.get, "function",
  "Function.prototype.arguments has function getter");
assert.sameValue(typeof argumentsDesc.set, "function",
  "Function.prototype.arguments has function setter");
assert.sameValue(argumentsDesc.get, argumentsDesc.set,
  "Function.prototype.arguments property getter/setter are the same function");

var throwTypeError;
WellKnownIntrinsicObjects.forEach(function(record) {
  if (record.name === "%ThrowTypeError%") {
    throwTypeError = record.value;
  }
});
if (throwTypeError) {
  assert.sameValue(argumentsDesc.set, throwTypeError, "Function.prototype.arguments getter is %ThrowTypeError%");
}
assert.throws(TypeError, function() {
  return Function.prototype.arguments;
});
assert.throws(TypeError, function() {
  Function.prototype.arguments = arguments;
});
