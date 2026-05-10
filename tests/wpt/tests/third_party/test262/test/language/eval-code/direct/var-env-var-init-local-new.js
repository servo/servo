// Copyright (C) 2016 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: sec-evaldeclarationinstantiation
description: Initialization of new variable binding
info: |
    [...]
    16. For each String vn in declaredVarNames, in list order do
        a. If varEnvRec is a global Environment Record, then
           [...]
        b. Else,
           i. Let bindingExists be varEnvRec.HasBinding(vn).
           ii. If bindingExists is false, then
               1. Let status be ! varEnvRec.CreateMutableBinding(vn, true).
               2. Assert: status is not an abrupt completion because of
                  validation preceding step 12.
               3. Perform ! varEnvRec.InitializeBinding(vn, undefined).
    [...]
flags: [noStrict]
---*/

var initial = null;
var postAssignment;

(function() {
  eval('initial = x; x = 4; postAssignment = x; var x;');
}());

assert.sameValue(initial, undefined);
assert.sameValue(postAssignment, 4, 'binding is mutable');
assert.throws(ReferenceError, function() {
  x;
});
