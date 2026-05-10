// Copyright (C) 2024 Justin Dorfman. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: sec-function.prototype.caller
description: >
  Function.prototype.caller property descriptor
info: |
  Function.prototype.caller is an accessor property whose set and get
  accessor functions are both %ThrowTypeError%.
includes: [propertyHelper.js, wellKnownIntrinsicObjects.js]
---*/

const callerDesc = Object.getOwnPropertyDescriptor(Function.prototype, 'caller');

verifyProperty(
  Function.prototype,
  "caller",
  { enumerable: false, configurable: true },
  { restore: true }
);

assert.sameValue(typeof callerDesc.get, "function",
  "Function.prototype.caller has function getter");
assert.sameValue(typeof callerDesc.set, "function",
  "Function.prototype.caller has function setter");
assert.sameValue(callerDesc.get, callerDesc.set,
  "Caller property getter/setter are the same function");

var throwTypeError;
WellKnownIntrinsicObjects.forEach(function(record) {
  if (record.name === "%ThrowTypeError%") {
    throwTypeError = record.value;
  }
});
if (throwTypeError) {
  assert.sameValue(callerDesc.set, throwTypeError, "Function.prototype.caller getter is %ThrowTypeError%");
}
assert.throws(TypeError, function() {
  return Function.prototype.caller;
});
assert.throws(TypeError, function fn() {
  Function.prototype.caller = fn;
});
