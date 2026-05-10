// This file was procedurally generated from the following sources:
// - src/annex-b-fns/global-existing-block-fn-no-init.case
// - src/annex-b-fns/global/switch-case.template
/*---
description: Does not re-initialize binding created by similar forms (Function declaration in the `case` clause of a `switch` statement in the global scope)
esid: sec-web-compat-globaldeclarationinstantiation
flags: [generated, noStrict]
info: |
    B.3.3.2 Changes to GlobalDeclarationInstantiation

    [...]
    b. If declaredFunctionOrVarNames does not contain F, then
       i. Perform ? envRec.CreateGlobalFunctionBinding(F, undefined, false).
       ii. Append F to declaredFunctionOrVarNames.
---*/
assert.sameValue(f, undefined);

{
  function f() {}
}

switch (1) {
  case 1:
    function f() {  }
}
