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
negative:
  phase: runtime
  type: SyntaxError
flags: [noStrict]
features: [let]
---*/

let x;

// Although the `try` statement is a more precise mechanism for detecting
// runtime errors, the behavior under test is only observable for a direct eval
// call when the call is made from the global scope. This forces the use of
// the more coarse-grained `negative` frontmatter to assert the expected error.

eval('var x;');
