// This file was procedurally generated from the following sources:
// - src/annex-b-fns/eval-func-existing-fn-update.case
// - src/annex-b-fns/eval-func/direct-block.template
/*---
description: Variable-scoped binding is updated following evaluation (Block statement in eval code containing a function declaration)
esid: sec-web-compat-evaldeclarationinstantiation
flags: [generated, noStrict]
info: |
    B.3.3.3 Changes to EvalDeclarationInstantiation

    [...]
    b. When the FunctionDeclaration f is evaluated, perform the following steps
       in place of the FunctionDeclaration Evaluation algorithm provided in
       14.1.21:
       i. Let genv be the running execution context's VariableEnvironment.
       ii. Let genvRec be genv's EnvironmentRecord.
       iii. Let benv be the running execution context's LexicalEnvironment.
       iv. Let benvRec be benv's EnvironmentRecord.
       v. Let fobj be ! benvRec.GetBindingValue(F, false).
       vi. Perform ? genvRec.SetMutableBinding(F, fobj, false).
       vii. Return NormalCompletion(empty). 
---*/
var after;

(function() {
  eval(
    '{ function f() { return "inner declaration"; } }after = f;\
    \
    function f() {\
      return "outer declaration";\
    }'
  );
}());

assert.sameValue(typeof after, 'function');
assert.sameValue(after(), 'inner declaration');
