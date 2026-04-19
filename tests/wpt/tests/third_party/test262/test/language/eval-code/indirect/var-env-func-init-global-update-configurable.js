// Copyright (C) 2016 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: sec-evaldeclarationinstantiation
description: Modification of previously-existing configurable global property
info: |
    [...]
    15. For each production f in functionsToInitialize, do
        a. Let fn be the sole element of the BoundNames of f.
        b. Let fo be the result of performing InstantiateFunctionObject for f
           with argument lexEnv.
        c. If varEnvRec is a global Environment Record, then
           i. Perform ? varEnvRec.CreateGlobalFunctionBinding(fn, fo, true).
    [...]

    8.1.1.4.18 CreateGlobalFunctionBinding

    [...]
    5. If existingProp is undefined or existingProp.[[Configurable]] is true,
       then
       a. Let desc be the PropertyDescriptor{[[Value]]: V, [[Writable]]: true,
          [[Enumerable]]: true, [[Configurable]]: D}.
    6. Else,
       [...]
    7. Perform ? DefinePropertyOrThrow(globalObject, N, desc).
    [...]
includes: [propertyHelper.js]
---*/

var initial = null;

Object.defineProperty(this, 'f', {
  enumerable: false,
  writable: false,
  configurable: true
});

(0, eval)('initial = f; function f() { return 345; }');

verifyProperty(this, 'f', {
  writable: true,
  enumerable: true,
  configurable: true,
});

assert.sameValue(typeof initial, 'function');
assert.sameValue(initial(), 345);
