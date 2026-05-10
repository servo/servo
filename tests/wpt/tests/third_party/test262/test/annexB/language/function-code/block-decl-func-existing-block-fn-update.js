// This file was procedurally generated from the following sources:
// - src/annex-b-fns/func-existing-block-fn-update.case
// - src/annex-b-fns/func/block.template
/*---
description: Variable-scoped binding is updated (Block statement in function scope containing a function declaration)
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
var updated;

(function() {
  {
    function f() {
      return 'first declaration';
    }
  }

  {
    function f() { return 'second declaration'; }
  }

  updated = f;
}());

assert.sameValue(typeof updated, 'function');
assert.sameValue(updated(), 'second declaration');
