// This file was procedurally generated from the following sources:
// - src/annex-b-fns/eval-global-existing-block-fn-no-init.case
// - src/annex-b-fns/eval-global/indirect-switch-case.template
/*---
description: Does not re-initialize binding created by similar forms (Function declaration in the `case` clause of a `switch` statement in eval code)
esid: sec-web-compat-evaldeclarationinstantiation
flags: [generated, noStrict]
info: |
    B.3.3.3 Changes to EvalDeclarationInstantiation

    [...]
    a. If declaredFunctionOrVarNames does not contain F, then
    [...]
---*/

(0,eval)(
  'assert.sameValue(f, undefined);\
  \
  {\
    function f() {}\
  }switch (1) {' +
  '  case 1:' +
  '    function f() {  }' +
  '}\
  '
);
