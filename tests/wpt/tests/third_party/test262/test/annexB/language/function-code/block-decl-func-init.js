// This file was procedurally generated from the following sources:
// - src/annex-b-fns/func-init.case
// - src/annex-b-fns/func/block.template
/*---
description: Variable binding is initialized to `undefined` in outer scope (Block statement in function scope containing a function declaration)
esid: sec-web-compat-functiondeclarationinstantiation
flags: [generated, noStrict]
info: |
    B.3.3.1 Changes to FunctionDeclarationInstantiation

    [...]
    2. If instantiatedVarNames does not contain F, then
       a. Perform ! varEnvRec.CreateMutableBinding(F, false).
       b. Perform varEnvRec.InitializeBinding(F, undefined).
       c. Append F to instantiatedVarNames.
    [...]
---*/
var init, changed;

(function() {
  init = f;
  f = 123;
  changed = f;

  {
    function f() {  }
  }

  
}());

assert.sameValue(init, undefined, 'binding is initialized to `undefined`');
assert.sameValue(changed, 123, 'binding is mutable');
assert.throws(ReferenceError, function() {
  f;
}, 'global binding is not created');
