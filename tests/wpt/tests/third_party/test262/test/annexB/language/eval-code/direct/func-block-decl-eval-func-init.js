// This file was procedurally generated from the following sources:
// - src/annex-b-fns/eval-func-init.case
// - src/annex-b-fns/eval-func/direct-block.template
/*---
description: Variable binding is initialized to `undefined` in outer scope (Block statement in eval code containing a function declaration)
esid: sec-web-compat-evaldeclarationinstantiation
flags: [generated, noStrict]
info: |
    B.3.3.3 Changes to EvalDeclarationInstantiation

    [...]
    a. If declaredFunctionOrVarNames does not contain F, then
       i. If varEnvRec is a global Environment Record, then
          [...]
       ii. Else,
           i. Let bindingExists be varEnvRec.HasBinding(F).
           ii. If bindingExists is false, then
               i. Perform ! varEnvRec.CreateMutableBinding(F, true).
               ii. Perform ! varEnvRec.InitializeBinding(F, undefined).
    [...]
---*/
var init, changed;

(function() {
  eval(
    'init = f;\
    f = 123;\
    changed = f;{ function f() {  } }'
  );
}());

assert.sameValue(init, undefined, 'binding is initialized to `undefined`');
assert.sameValue(changed, 123, 'binding is mutable');
assert.throws(ReferenceError, function() {
  f;
}, 'global binding is not created');
