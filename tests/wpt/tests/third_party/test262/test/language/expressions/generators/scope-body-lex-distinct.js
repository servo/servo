// Copyright (C) 2016 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: sec-functiondeclarationinstantiation
description: >
    Creation of new lexical environment (distinct from the variable
    environment) for the function body outside of strict mode
info: |
    [...]
    29. If strict is false, then
        a. Let lexEnv be NewDeclarativeEnvironment(varEnv).
        b. NOTE: Non-strict functions use a separate lexical Environment Record
           for top-level lexical declarations so that a direct eval can
           determine whether any var scoped declarations introduced by the eval
           code conflict with pre-existing top-level lexically scoped
           declarations.  This is not needed for strict functions because a
           strict direct eval always places all declarations into a new
           Environment Record.
    [...]

    18.2.1.3 Runtime Semantics: EvalDeclarationInstantiation

    [...]
    5. If strict is false, then
       [...]
       b. Let thisLex be lexEnv.
       c. Assert: The following loop will terminate.
       d. Repeat while thisLex is not the same as varEnv,
          i. Let thisEnvRec be thisLex's EnvironmentRecord.
          ii. If thisEnvRec is not an object Environment Record, then
              1. NOTE: The environment of with statements cannot contain any
                 lexical declaration so it doesn't need to be checked for
                 var/let hoisting conflicts.
              2. For each name in varNames, do
                 a. If thisEnvRec.HasBinding(name) is true, then
                    i. Throw a SyntaxError exception.
                    ii. NOTE: Annex B.3.5 defines alternate semantics for the
                        above step.
                 b. NOTE: A direct eval will not hoist var declaration over a
                    like-named lexical declaration.
          iii. Let thisLex be thisLex's outer environment reference.
flags: [noStrict]
features: [generators, let]
---*/

var g = function*() {
  let x;
  eval('var x;');
};
var iter = g();

assert.throws(SyntaxError, function() {
  iter.next();
});
