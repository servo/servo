// This file was procedurally generated from the following sources:
// - src/annex-b-fns/eval-global-existing-var-no-init.case
// - src/annex-b-fns/eval-global/indirect-switch-dflt.template
/*---
description: Existing variable binding is not modified (Funtion declaration in the `default` clause of a `switch` statement in eval code in the global scope)
esid: sec-web-compat-evaldeclarationinstantiation
flags: [generated, noStrict]
info: |
    B.3.3.3 Changes to EvalDeclarationInstantiation

    [...]
    a. If declaredFunctionOrVarNames does not contain F, then
    [...]
---*/

(0,eval)(
  'var f = 123;\
  assert.sameValue(f, 123);switch (1) {' +
  '  default:' +
  '    function f() {  }' +
  '}\
  '
);
