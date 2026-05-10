// This file was procedurally generated from the following sources:
// - src/annex-b-fns/eval-global-existing-block-fn-no-init.case
// - src/annex-b-fns/eval-global/direct-block.template
/*---
description: Does not re-initialize binding created by similar forms (Block statement in eval code containing a function declaration)
esid: sec-web-compat-evaldeclarationinstantiation
flags: [generated, noStrict]
info: |
    B.3.3.3 Changes to EvalDeclarationInstantiation

    [...]
    a. If declaredFunctionOrVarNames does not contain F, then
    [...]
---*/

eval(
  'assert.sameValue(f, undefined);\
  \
  {\
    function f() {}\
  }{ function f() {  } }'
);
