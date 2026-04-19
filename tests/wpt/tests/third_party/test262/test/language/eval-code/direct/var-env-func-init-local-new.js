// Copyright (C) 2016 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: sec-evaldeclarationinstantiation
description: Initialization of new local binding
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
               1. Let status be ! varEnvRec.CreateMutableBinding(fn, true).
               2. Assert: status is not an abrupt completion because of
                  validation preceding step 12.
               3. Perform ! varEnvRec.InitializeBinding(fn, fo).
           [...]
flags: [noStrict]
---*/

var initial, postAssignment;
(function() {
  eval('initial = f; f = 5; postAssignment = f; function f() { return 33; }');
}());

assert.sameValue(typeof initial, 'function');
assert.sameValue(initial(), 33);
assert.sameValue(postAssignment, 5, 'binding is mutable');
assert.throws(ReferenceError, function() {
  f;
});
