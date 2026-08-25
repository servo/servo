// Copyright (C) 2016 André Bargull. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-%throwtypeerror%
description: >
  The value of the [[Prototype]] internal slot of %ThrowTypeError% is the
  intrinsic object %FunctionPrototype%.
info: |
  8.2.2 CreateIntrinsics ( realmRec )

  ...
  6. Let thrower be CreateBuiltinFunction(realmRec, throwerSteps, null).
  7. Set intrinsics.[[%ThrowTypeError%]] to thrower.
  8. Let noSteps be an empty sequence of algorithm steps.
  9. Let funcProto be CreateBuiltinFunction(realmRec, noSteps, objProto).
  10. Set intrinsics.[[%FunctionPrototype%]] to funcProto.
  11. Call thrower.[[SetPrototypeOf]](funcProto).
  ...
---*/

var ThrowTypeError = Object.getOwnPropertyDescriptor(function() {
  "use strict";
  return arguments;
}(), "callee").get;

assert.sameValue(Object.getPrototypeOf(ThrowTypeError), Function.prototype);
