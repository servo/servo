// This file was procedurally generated from the following sources:
// - src/annex-b-fns/eval-global-existing-non-enumerable-global-init.case
// - src/annex-b-fns/eval-global/direct-if-decl-no-else.template
/*---
description: Variable binding is left in place by legacy function hoisting. CreateGlobalVariableBinding leaves the binding as non-enumerable even if it has the chance to change it to be enumerable. (IfStatement without an else clause in eval code)
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
  }, { restore: true });if (true) function f() {  }'
);

assert.sameValue(typeof f, "function");
verifyProperty(global, 'f', {
  enumerable: false,
  writable: true,
  configurable: true
});
