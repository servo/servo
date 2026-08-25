// This file was procedurally generated from the following sources:
// - src/annex-b-fns/global-init.case
// - src/annex-b-fns/global/if-decl-no-else.template
/*---
description: Variable binding is initialized to `undefined` in outer scope (IfStatement without an else clause in the global scope)
esid: sec-functiondeclarations-in-ifstatement-statement-clauses
flags: [generated, noStrict]
includes: [fnGlobalObject.js, propertyHelper.js]
info: |
    The following rules for IfStatement augment those in 13.6:

    IfStatement[Yield, Return]:
        if ( Expression[In, ?Yield] ) FunctionDeclaration[?Yield] else Statement[?Yield, ?Return]
        if ( Expression[In, ?Yield] ) Statement[?Yield, ?Return] else FunctionDeclaration[?Yield]
        if ( Expression[In, ?Yield] ) FunctionDeclaration[?Yield] else FunctionDeclaration[?Yield]
        if ( Expression[In, ?Yield] ) FunctionDeclaration[?Yield]


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

if (true) function f() {  }
