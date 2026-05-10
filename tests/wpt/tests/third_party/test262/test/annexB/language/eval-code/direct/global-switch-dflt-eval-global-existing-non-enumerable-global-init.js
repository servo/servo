// This file was procedurally generated from the following sources:
// - src/annex-b-fns/eval-global-existing-non-enumerable-global-init.case
// - src/annex-b-fns/eval-global/direct-switch-dflt.template
/*---
description: Variable binding is left in place by legacy function hoisting. CreateGlobalVariableBinding leaves the binding as non-enumerable even if it has the chance to change it to be enumerable. (Funtion declaration in the `default` clause of a `switch` statement in eval code in the global scope)
esid: sec-web-compat-evaldeclarationinstantiation
flags: [generated, noStrict]
includes: [fnGlobalObject.js, propertyHelper.js]
info: |
    B.3.3.3 Changes to EvalDeclarationInstantiation

    [...]
    i. If varEnvRec is a global Environment Record, then
       i. Perform ? varEnvRec.CreateGlobalVarBinding(F, true).
    [...]

---*/
Object.defineProperty(fnGlobalObject(), 'f', {
  value: 'x',
  enumerable: false,
  writable: true,
  configurable: true
});

eval(
  'var global = fnGlobalObject();\
  assert.sameValue(f, "x", "binding is not reinitialized");\
  \
  verifyProperty(global, "f", {\
    enumerable: false,\
    writable: true,\
    configurable: true\
  }, { restore: true });switch (1) {' +
  '  default:' +
  '    function f() {  }' +
  '}\
  '
);

assert.sameValue(typeof f, "function");
verifyProperty(global, 'f', {
  enumerable: false,
  writable: true,
  configurable: true
});
