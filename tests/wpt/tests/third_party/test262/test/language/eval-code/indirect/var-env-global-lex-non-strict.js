// Copyright (C) 2016 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: sec-evaldeclarationinstantiation
description: Variable collision with global lexical binding
info: |
    [...]
    5. If strict is false, then
       a. If varEnvRec is a global Environment Record, then
          i. For each name in varNames, do
             1. If varEnvRec.HasLexicalDeclaration(name) is true, throw a
                SyntaxError exception.
             2. NOTE: eval will not create a global var declaration that would
                be shadowed by a global lexical declaration.
       [...]
features: [let]
---*/

let x;
var caught;

// The `assert.throws` helper function would interfere with the semantics under
// test.
try {
  (0,eval)('var x;');
} catch (err) {
  caught = err;
}

assert.notSameValue(caught, undefined);
assert.sameValue(caught.constructor, SyntaxError);
