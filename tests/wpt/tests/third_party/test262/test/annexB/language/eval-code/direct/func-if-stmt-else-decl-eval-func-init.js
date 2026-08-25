// This file was procedurally generated from the following sources:
// - src/annex-b-fns/eval-func-init.case
// - src/annex-b-fns/eval-func/direct-if-stmt-else-decl.template
/*---
description: Variable binding is initialized to `undefined` in outer scope (IfStatement with a declaration in the second statement position in eval code)
esid: sec-functiondeclarations-in-ifstatement-statement-clauses
flags: [generated, noStrict]
info: |
    The following rules for IfStatement augment those in 13.6:

    IfStatement[Yield, Return]:
        if ( Expression[In, ?Yield] ) FunctionDeclaration[?Yield] else Statement[?Yield, ?Return]
        if ( Expression[In, ?Yield] ) Statement[?Yield, ?Return] else FunctionDeclaration[?Yield]
        if ( Expression[In, ?Yield] ) FunctionDeclaration[?Yield] else FunctionDeclaration[?Yield]
        if ( Expression[In, ?Yield] ) FunctionDeclaration[?Yield]


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
    changed = f;if (false) ; else function f() {  }'
  );
}());

assert.sameValue(init, undefined, 'binding is initialized to `undefined`');
assert.sameValue(changed, 123, 'binding is mutable');
assert.throws(ReferenceError, function() {
  f;
}, 'global binding is not created');
