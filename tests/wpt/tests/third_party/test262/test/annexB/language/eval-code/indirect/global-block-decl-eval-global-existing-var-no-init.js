// This file was procedurally generated from the following sources:
// - src/annex-b-fns/eval-global-existing-var-no-init.case
// - src/annex-b-fns/eval-global/indirect-block.template
/*---
description: Existing variable binding is not modified (Block statement in eval code containing a function declaration)
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
  assert.sameValue(f, 123);{ function f() {  } }'
);
