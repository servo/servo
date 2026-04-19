// Copyright (C) 2016 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: sec-evaldeclarationinstantiation
description: Declaration does not modify existing global property
info: |
    [...]
    16. For each String vn in declaredVarNames, in list order do
        a. If varEnvRec is a global Environment Record, then
           i. Perform ? varEnvRec.CreateGlobalVarBinding(vn, true).
    [...]

    8.1.1.4.17 CreateGlobalVarBinding

    [...]
    5. Let extensible be ? IsExtensible(globalObject).
    6. If hasProperty is false and extensible is true, then
       [...]
    [...]
includes: [propertyHelper.js]
---*/

var initial;
var x = 23;

(0, eval)('initial = x; var x = 45;');

verifyProperty(this, 'x', {
  value: 45,
  writable: true,
  enumerable: true,
  configurable: false,
});

assert.sameValue(initial, 23);
