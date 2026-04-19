// This file was procedurally generated from the following sources:
// - src/annex-b-fns/global-existing-non-enumerable-global-init.case
// - src/annex-b-fns/global/switch-dflt.template
/*---
description: Variable binding is left in place by legacy function hoisting. CreateGlobalVariableBinding leaves the binding as non-enumerable even if it has the chance to change it to be enumerable. (Funtion declaration in the `default` clause of a `switch` statement in the global scope)
esid: sec-web-compat-globaldeclarationinstantiation
flags: [generated, noStrict]
includes: [fnGlobalObject.js, propertyHelper.js]
info: |
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

switch (1) {
  default:
    function f() { return 'inner declaration'; }
}

`);

$262.evalScript(`
assert.sameValue(f(), 'inner declaration');
verifyProperty(global, 'f', {
  enumerable: false,
  writable: true,
  configurable: true
});
`);
