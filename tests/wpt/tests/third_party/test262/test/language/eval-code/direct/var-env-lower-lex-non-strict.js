// Copyright (C) 2016 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: sec-evaldeclarationinstantiation
description: Variable collision with lexical binding in lower scope
info: |
    [...]
    5. If strict is false, then
       [...]
       d. Repeat while thisLex is not the same as varEnv,
          i. Let thisEnvRec be thisLex's EnvironmentRecord.
          ii. If thisEnvRec is not an object Environment Record, then
              1. NOTE: The environment of with statements cannot contain any
                 lexical declaration so it doesn't need to be checked for
                 var/let hoisting conflicts.
              2. For each name in varNames, do
                 a. If thisEnvRec.HasBinding(name) is true, then
                    i. Throw a SyntaxError exception.
                 b. NOTE: A direct eval will not hoist var declaration over a
                    like-named lexical declaration.
          iii. Let thisLex be thisLex's outer environment reference.
    [...]
flags: [noStrict]
features: [let]
---*/

assert.throws(SyntaxError, function() {
  {
    let x;
    {
      eval('var x;');
    }
  }
});
