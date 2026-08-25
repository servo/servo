// This file was procedurally generated from the following sources:
// - src/annex-b-fns/eval-global-existing-var-no-init.case
// - src/annex-b-fns/eval-global/direct-switch-case.template
/*---
description: Existing variable binding is not modified (Function declaration in the `case` clause of a `switch` statement in eval code)
esid: sec-web-compat-evaldeclarationinstantiation
flags: [generated, noStrict]
info: |
    B.3.3.3 Changes to EvalDeclarationInstantiation

    [...]
    a. If declaredFunctionOrVarNames does not contain F, then
    [...]
---*/

eval(
  'var f = 123;\
  assert.sameValue(f, 123);switch (1) {' +
  '  case 1:' +
  '    function f() {  }' +
  '}\
  '
);
