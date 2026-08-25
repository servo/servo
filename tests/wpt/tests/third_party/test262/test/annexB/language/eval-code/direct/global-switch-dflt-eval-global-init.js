// This file was procedurally generated from the following sources:
// - src/annex-b-fns/eval-global-init.case
// - src/annex-b-fns/eval-global/direct-switch-dflt.template
/*---
description: Variable binding is initialized to `undefined` in outer scope (Funtion declaration in the `default` clause of a `switch` statement in eval code in the global scope)
esid: sec-web-compat-evaldeclarationinstantiation
flags: [generated, noStrict]
includes: [fnGlobalObject.js, propertyHelper.js]
info: |
    B.3.3.3 Changes to EvalDeclarationInstantiation

    [...]
    i. If varEnvRec is a global Environment Record, then
       i. Perform ? varEnvRec.CreateGlobalFunctionBinding(F, undefined, true).
    [...]

---*/

eval(
  'var global = fnGlobalObject();\
  assert.sameValue(f, undefined, "binding is initialized to `undefined`");\
  \
  verifyProperty(global, "f", {\
    enumerable: true,\
    writable: true,\
    configurable: true\
  });switch (1) {' +
  '  default:' +
  '    function f() {  }' +
  '}\
  '
);
