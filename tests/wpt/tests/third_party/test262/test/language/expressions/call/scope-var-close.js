// Copyright (C) 2016 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: sec-prepareforordinarycall
description: >
    Removal of variable environment for the function parameters and body
info: |
    [...]
    3. Let callerContext be the running execution context.
    [...]
    8. Remove calleeContext from the execution context stack and restore
       callerContext as the running execution context.
    [...]
---*/

var probe;

// This test intentionally elides parameter expressions because their presence
// triggers the creation of an additional LexicalEnvironment dedicated to the
// function body (see sec-functiondeclarationinstantiation)
(function() {
  var x = 'inside';
  probe = function() { return x; };
}());

var x = 'outside';

assert.sameValue(probe(), 'inside');
assert.sameValue(x, 'outside');
