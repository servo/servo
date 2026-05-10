// This file was procedurally generated from the following sources:
// - src/annex-b-fns/global-init.case
// - src/annex-b-fns/global/block.template
/*---
description: Variable binding is initialized to `undefined` in outer scope (Block statement in the global scope containing a function declaration)
esid: sec-web-compat-globaldeclarationinstantiation
flags: [generated, noStrict]
includes: [fnGlobalObject.js, propertyHelper.js]
info: |
    B.3.3.2 Changes to GlobalDeclarationInstantiation

    [...]
    b. If declaredFunctionOrVarNames does not contain F, then
       i. Perform ? envRec.CreateGlobalFunctionBinding(F, undefined, false).
       ii. Append F to declaredFunctionOrVarNames.
    [...]

---*/
var global = fnGlobalObject();
assert.sameValue(f, undefined, 'binding is initialized to `undefined`');

verifyProperty(global, "f", {
  enumerable: true,
  writable: true,
  configurable: false
});

{
  function f() {  }
}
