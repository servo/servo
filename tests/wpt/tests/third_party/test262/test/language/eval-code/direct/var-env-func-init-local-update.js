// Copyright (C) 2016 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: sec-evaldeclarationinstantiation
description: Re-declaration of an existing local variable binding
info: |
    [...]
    15. For each production f in functionsToInitialize, do
        a. Let fn be the sole element of the BoundNames of f.
        b. Let fo be the result of performing InstantiateFunctionObject for f
           with argument lexEnv.
        c. If varEnvRec is a global Environment Record, then
           [...]
        d. Else,
           i. Let bindingExists be varEnvRec.HasBinding(fn).
           ii. If bindingExists is false, then
               [...]
           iii. Else,
                1. Perform ! varEnvRec.SetMutableBinding(fn, fo, false).
    [...]
flags: [noStrict]
---*/

var initial;

(function() {
  var f = 88;
  eval('initial = f; function f() { return 33; }');
}());

assert.sameValue(typeof initial, 'function');
assert.sameValue(initial(), 33);
assert.throws(ReferenceError, function() {
  f;
});
