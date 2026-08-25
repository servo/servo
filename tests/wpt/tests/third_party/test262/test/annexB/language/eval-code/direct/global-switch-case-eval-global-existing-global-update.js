// This file was procedurally generated from the following sources:
// - src/annex-b-fns/eval-global-existing-global-update.case
// - src/annex-b-fns/eval-global/direct-switch-case.template
/*---
description: Variable-scoped binding is updated following evaluation (Function declaration in the `case` clause of a `switch` statement in eval code)
esid: sec-web-compat-evaldeclarationinstantiation
flags: [generated, noStrict]
includes: [fnGlobalObject.js]
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
Object.defineProperty(fnGlobalObject(), 'f', {
  value: function() { return 'Another function'; },
  enumerable: true,
  writable: true,
  configurable: false
});

eval(
  'switch (1) {' +
  '  case 1:' +
  '    function f() { return "function declaration"; }' +
  '}\
  '
);

assert.sameValue(typeof f, 'function');
assert.sameValue(f(), 'function declaration');
