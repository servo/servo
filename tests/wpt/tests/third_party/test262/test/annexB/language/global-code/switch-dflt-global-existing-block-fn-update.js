// This file was procedurally generated from the following sources:
// - src/annex-b-fns/global-existing-block-fn-update.case
// - src/annex-b-fns/global/switch-dflt.template
/*---
description: Variable-scoped binding is updated (Funtion declaration in the `default` clause of a `switch` statement in the global scope)
esid: sec-web-compat-globaldeclarationinstantiation
flags: [generated, noStrict]
info: |
    B.3.3.2 Changes to GlobalDeclarationInstantiation
    [...]
    c. When the FunctionDeclaration f is evaluated, perform the following steps
       in place of the FunctionDeclaration Evaluation algorithm provided in
       14.1.21:

       i. Let genv be the running execution context's VariableEnvironment.
       ii. Let genvRec be genv's EnvironmentRecord.
       ii. Let benv be the running execution context's LexicalEnvironment.
       iv. Let benvRec be benv's EnvironmentRecord.
       v. Let fobj be ! benvRec.GetBindingValue(F, false).
       vi. Perform ? genvRec.SetMutableBinding(F, fobj, false).
       vii. Return NormalCompletion(empty).
---*/
{
  function f() {
    return 'first declaration';
  }
}

switch (1) {
  default:
    function f() { return 'second declaration'; }
}

assert.sameValue(typeof f, 'function');
assert.sameValue(f(), 'second declaration');
