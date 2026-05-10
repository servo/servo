// This file was procedurally generated from the following sources:
// - src/annex-b-fns/eval-global-existing-global-init.case
// - src/annex-b-fns/eval-global/indirect-switch-dflt.template
/*---
description: Variable binding is left in place by legacy function hoisting (Funtion declaration in the `default` clause of a `switch` statement in eval code in the global scope)
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
  enumerable: true,
  writable: true,
  configurable: false
});

(0,eval)(
  'var global = fnGlobalObject();\
  assert.sameValue(f, "x", "binding is not reinitialized");\
  \
  verifyProperty(global, "f", {\
    enumerable: true,\
    writable: true,\
    configurable: false\
  }, { restore: true });switch (1) {' +
  '  default:' +
  '    function f() {  }' +
  '}\
  '
);

assert.sameValue(typeof f, "function");
verifyProperty(global, "f", {
  enumerable: true,
  writable: true,
  configurable: false
});
