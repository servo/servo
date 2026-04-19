// This file was procedurally generated from the following sources:
// - src/annex-b-fns/func-existing-fn-update.case
// - src/annex-b-fns/func/switch-case.template
/*---
description: Variable-scoped binding is updated following evaluation (Function declaration in the `case` clause of a `switch` statement in function scope)
esid: sec-web-compat-functiondeclarationinstantiation
flags: [generated, noStrict]
info: |
    B.3.3.1 Changes to FunctionDeclarationInstantiation

    [...]
    3. When the FunctionDeclaration f is evaluated, perform the following steps
       in place of the FunctionDeclaration Evaluation algorithm provided in
       14.1.21:
       a. Let fenv be the running execution context's VariableEnvironment.
       b. Let fenvRec be fenv's EnvironmentRecord.
       c. Let benv be the running execution context's LexicalEnvironment.
       d. Let benvRec be benv's EnvironmentRecord.
       e. Let fobj be ! benvRec.GetBindingValue(F, false).
       f. Perform ! fenvRec.SetMutableBinding(F, fobj, false).
       g. Return NormalCompletion(empty). 
---*/
var after;

(function() {
  

  switch (1) {
    case 1:
      function f() { return 'inner declaration'; }
  }

  after = f;

  function f() {
    return 'outer declaration';
  }
}());

assert.sameValue(typeof after, 'function');
assert.sameValue(after(), 'inner declaration');
