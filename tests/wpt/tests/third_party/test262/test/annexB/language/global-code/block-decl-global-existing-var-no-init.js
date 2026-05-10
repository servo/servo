// This file was procedurally generated from the following sources:
// - src/annex-b-fns/global-existing-var-no-init.case
// - src/annex-b-fns/global/block.template
/*---
description: Existing variable binding is not modified (Block statement in the global scope containing a function declaration)
esid: sec-web-compat-globaldeclarationinstantiation
flags: [generated, noStrict]
info: |
    B.3.3.2 Changes to GlobalDeclarationInstantiation

    [...]
    b. If declaredFunctionOrVarNames does not contain F, then
       i. Perform ? envRec.CreateGlobalFunctionBinding(F, undefined, false).
       ii. Append F to declaredFunctionOrVarNames.
    [...]
---*/
var f = 123;
assert.sameValue(f, 123);

{
  function f() {  }
}
