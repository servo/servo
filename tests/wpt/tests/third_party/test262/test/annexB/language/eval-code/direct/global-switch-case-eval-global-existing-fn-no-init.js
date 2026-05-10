// This file was procedurally generated from the following sources:
// - src/annex-b-fns/eval-global-existing-fn-no-init.case
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
  'assert.sameValue(f(), "outer declaration");switch (1) {' +
  '  case 1:' +
  '    function f() { return "inner declaration"; }' +
  '}\
  function f() {\
    return "outer declaration";\
  }'
);
