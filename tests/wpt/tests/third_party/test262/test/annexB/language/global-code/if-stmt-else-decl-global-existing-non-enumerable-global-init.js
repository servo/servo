// This file was procedurally generated from the following sources:
// - src/annex-b-fns/global-existing-non-enumerable-global-init.case
// - src/annex-b-fns/global/if-stmt-else-decl.template
/*---
description: Variable binding is left in place by legacy function hoisting. CreateGlobalVariableBinding leaves the binding as non-enumerable even if it has the chance to change it to be enumerable. (IfStatement with a declaration in the second statement position in the global scope)
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


    B.3.3.3 Changes to GlobalDeclarationInstantiation

    [...]
    Perform ? varEnvRec.CreateGlobalVarBinding(F, true).
    [...]

---*/
var global = fnGlobalObject();
Object.defineProperty(global, 'f', {
  value: 'x',
  enumerable: false,
  writable: true,
  configurable: true
});

$262.evalScript(`
assert.sameValue(f, 'x');
verifyProperty(global, 'f', {
  enumerable: false,
  writable: true,
  configurable: true
}, { restore: true });
`);

$262.evalScript(`

if (false) ; else function f() { return 'inner declaration'; }

`);

$262.evalScript(`
assert.sameValue(f(), 'inner declaration');
verifyProperty(global, 'f', {
  enumerable: false,
  writable: true,
  configurable: true
});
`);
