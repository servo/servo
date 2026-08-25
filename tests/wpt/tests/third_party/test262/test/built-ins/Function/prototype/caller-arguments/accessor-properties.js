// Copyright (C) 2024 Justin Dorfman. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: sec-addrestrictedfunctionproperties
description: >
  Function.prototype.arguments and Function.prototype.arguments are both
  accessor properties whose set and get functions are both %ThrowTypeError%.
info: |
  2. Let _thrower_ be _realm_.[[Intrinsics]].[[%ThrowTypeError%]].
  3. Perform ! DefinePropertyOrThrow(_F_, *"caller"*, PropertyDescriptor { [[Get]]: _thrower_, [[Set]]: _thrower_, [[Enumerable]]: *false*, [[Configurable]]: *true* }).
  4. Perform ! DefinePropertyOrThrow(_F_, *"arguments"*, PropertyDescriptor { [[Get]]: _thrower_, [[Set]]: _thrower_, [[Enumerable]]: *false*, [[Configurable]]: *true* }).
---*/

const callerDesc = Object.getOwnPropertyDescriptor(Function.prototype, "caller");
const argumentsDesc = Object.getOwnPropertyDescriptor(Function.prototype, "arguments");

// Other tests at ../{arguments,caller}/prop-desc.js already assert that each
// getter/setter pair use a single function (and when possible, that the
// function is %ThrowTypeError%), so this test only needs to assert equality
// *across* the pairs.
assert.sameValue(callerDesc.get, argumentsDesc.get,
  "Function.prototype.arguments and Function.prototype.caller accessor functions should match (%ThrowTypeError%)");
